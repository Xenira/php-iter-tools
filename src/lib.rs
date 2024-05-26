use std::ptr;

use exceptions::IterError;
use ext_php_rs::{
    boxed::ZBox,
    convert::{FromZval, IntoZval},
    ffi::{
        self, zend_call_function, zend_fcall_info_init, zend_hash_get_current_data_ex, zval,
        HashPosition,
    },
    flags::DataType,
    prelude::*,
    types::{ZendClassObject, ZendHashTable, Zval},
};
use macros::{match_iter_result_type, match_iter_same_type, match_nested_iter_type};

use crate::macros::match_iter_type;

mod exceptions;
mod macros;

pub struct SimpleZValIter {
    inner: Zval,
    size: HashPosition,
    pos_front: HashPosition,
    pos_back: HashPosition,
}

impl Iterator for SimpleZValIter {
    type Item = ZVal;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos_front == self.pos_back {
            return None;
        }

        let val = unsafe {
            &*zend_hash_get_current_data_ex(
                self.inner.value.arr as *const ZendHashTable as *mut ZendHashTable,
                &mut self.pos_front as &mut _,
            )
        };

        if ptr::null() == val {
            return None;
        }

        self.pos_front += 1;

        Some(ZVal::from(val))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let left = self.pos_back - self.pos_front;
        (left as usize, Some(left as usize))
    }
}

impl ExactSizeIterator for SimpleZValIter {
    fn len(&self) -> usize {
        (self.pos_back - self.pos_front) as usize
    }
}

impl DoubleEndedIterator for SimpleZValIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.pos_back == self.pos_front {
            return None;
        }

        self.pos_back -= 1;
        let val = unsafe {
            &*zend_hash_get_current_data_ex(
                self.inner.value.arr as *const ZendHashTable as *mut ZendHashTable,
                &mut self.pos_back as &mut _,
            )
        };
        if ptr::null() == val {
            return None;
        }

        Some(ZVal::from(val))
    }
}

impl Clone for SimpleZValIter {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.shallow_clone(),
            size: self.size,
            pos_front: self.pos_front,
            pos_back: self.pos_back,
        }
    }
}

/// Blazingly 🔥 fast iterator for PHP arrays.
///
/// @template T
#[php_class(name = "Iter")]
pub struct ArrayIterator {
    iter: Option<IterBox<'static>>,
}

impl IntoIterator for ZVal {
    type Item = ZVal;
    type IntoIter = SimpleZValIter;

    fn into_iter(self) -> Self::IntoIter {
        let array = self.inner.array().unwrap();
        let pos_front = 0;
        let size = array.nNumOfElements;
        let pos_back = size;
        return SimpleZValIter {
            inner: self.inner,
            size,
            pos_front,
            pos_back,
        };
    }
}

#[php_impl]
impl ArrayIterator {
    /// Creates a new iterator from an array.
    ///
    /// @param list<T> $vec
    pub fn new(vec: &Zval) -> Self {
        Self {
            iter: Some(IterBox::DoubleEndedExactSize(Box::new(
                ZVal::from(vec).into_iter(),
            ))),
        }
    }

    /// Advances the iterator and returns the next value.
    ///
    /// Returns `null` when iteration is finished. Individual iterator implementations may choose to resume iteration, and so calling next() again may or may not eventually start returning values again at some point.
    ///
    /// @return T|null
    pub fn next(&mut self) -> Result<Option<Zval>, IterError> {
        self.iter.as_mut().map_or(Err(IterError::Consumed), |iter| {
            Ok(match_iter_type!(
                iter,
                iter.next().map(|x| x.inner),
                IterBox::DoubleEndedExactSize
                    | IterBox::DoubleEnded
                    | IterBox::ExactSize
                    | IterBox::Iterator
            ))
        })
    }

    /// Returns the bounds on the remaining length of the iterator.
    ///
    /// Specifically, size_hint() returns a tuple where the first element is the lower bound, and the second element is the upper bound.
    ///
    /// The second half of the tuple that is returned is an Option<usize>. A None here means that either there is no known upper bound, or the upper bound is larger than usize.
    ///
    /// # Implementation notes
    /// It is not enforced that an iterator implementation yields the declared number of elements. A buggy iterator may yield less than the lower bound or more than the upper bound of elements.
    ///
    /// size_hint() is primarily intended to be used for optimizations such as reserving space for the elements of the iterator, but must not be trusted to e.g., omit bounds checks in unsafe code. An incorrect implementation of size_hint() should not lead to memory safety violations.
    ///
    /// That said, the implementation should provide a correct estimation, because otherwise it would be a violation of the trait’s protocol.
    ///
    /// The default implementation returns (0, None) which is correct for any iterator.
    ///
    /// @return array{int, int|null}
    /// @throws \LogicException
    /// @throws \Exception
    pub fn size_hint(&mut self) -> Result<ZBox<ZendHashTable>, IterError> {
        self.iter.as_mut().map_or(Err(IterError::Consumed), |iter| {
            let size_hint = match_iter_type!(
                iter,
                iter.size_hint(),
                IterBox::DoubleEndedExactSize
                    | IterBox::DoubleEnded
                    | IterBox::ExactSize
                    | IterBox::Iterator
            );

            let (lower, upper) = size_hint;
            let mut map = ZendHashTable::new();
            map.push(lower);
            map.push(
                upper
                    .map(|upper| {
                        let mut val = Zval::new();
                        val.set_long(upper as i64);
                        val
                    })
                    .unwrap_or(Zval::new()),
            );

            Ok(map)
        })
    }

    /// Consumes the iterator, counting the number of iterations and returning it.
    ///
    /// This method will call next repeatedly until `null` is encountered, returning the number of times it saw a value. Note that next has to be called at least once even if the iterator does not have any elements.
    ///
    /// # Overflow Behavior
    /// The method does no guarding against overflows, so counting elements of an iterator with more than `usize::MAX` elements either produces the wrong result or panics. If debug assertions are enabled, a panic is guaranteed.
    ///
    /// # Panics
    /// This function might panic if the iterator has more than `usize::MAX` elements.
    pub fn count(&mut self) -> Result<i64, IterError> {
        self.iter.as_mut().map_or(Err(IterError::Consumed), |iter| {
            Ok(match_iter_type!(
                iter,
                iter.count() as i64,
                IterBox::DoubleEndedExactSize
                    | IterBox::DoubleEnded
                    | IterBox::ExactSize
                    | IterBox::Iterator
            ))
        })
    }

    /// Consumes the iterator, returning the last element.
    ///
    /// This method will evaluate the iterator until it returns `null`. While doing so, it keeps track of the current element. After `null` is returned, `last()` will then return the last element it saw.
    ///
    /// @return T|null
    pub fn last(&mut self) -> Result<Option<Zval>, IterError> {
        self.iter.as_mut().map_or(Err(IterError::Consumed), |iter| {
            Ok(match_iter_type!(
                iter,
                iter.last().map(|x| x.inner),
                IterBox::DoubleEndedExactSize
                    | IterBox::DoubleEnded
                    | IterBox::ExactSize
                    | IterBox::Iterator
            ))
        })
    }

    /// Returns the nth element of the iterator.
    ///
    /// Like most indexing operations, the count starts from zero, so `nth(0)` returns the first value, `nth(1)` the second, and so on.
    ///
    /// Note that all preceding elements, as well as the returned element, will be consumed from the iterator. That means that the preceding elements will be discarded, and also that calling `nth(0)` multiple times on the same iterator will return different elements.
    ///
    /// `nth()` will return `null` if `n` is greater than or equal to the length of the iterator.
    ///
    /// @return T|null
    pub fn nth(&mut self, n: i64) -> Result<Option<Zval>, IterError> {
        self.iter.as_mut().map_or(Err(IterError::Consumed), |iter| {
            Ok(match_iter_type!(
                iter,
                iter.nth(n as usize).map(|x| x.inner),
                IterBox::DoubleEndedExactSize
                    | IterBox::DoubleEnded
                    | IterBox::ExactSize
                    | IterBox::Iterator
            ))
        })
    }

    /// Creates an iterator starting at the same point, but stepping by the given amount at each iteration.
    ///
    /// Note 1: The first element of the iterator will always be returned, regardless of the step given.
    ///
    /// Note 2: The time at which ignored elements are pulled is not fixed. StepBy behaves like the sequence `self.next()`, `self.nth(step-1)`, `self.nth(step-1)`, …, but is also free to behave like the sequence `advance_n_and_return_first(&mut self, step)`, `advance_n_and_return_first(&mut self, step)`, … Which way is used may change for some iterators for performance reasons. The second way will advance the iterator earlier and may consume more items.
    ///
    /// # Panics
    /// The method will panic if the given step is 0.
    ///
    /// @param positive-int $step
    /// @return self<T>
    /// @throws \LogicException
    /// @throws \ValueError
    pub fn step_by(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        step: i64,
    ) -> Result<&mut ZendClassObject<ArrayIterator>, IterError> {
        if step == 0 {
            return Err(IterError::ArgumentError(format!(
                "Step must be greater than 0, {step} given"
            )));
        }

        let iter = this.iter.take().ok_or(IterError::Moved)?;
        this.iter = Some(match_iter_result_type!(
            iter,
            Box::new(iter.step_by(step as usize)),
            IterBox::DoubleEndedExactSize => IterBox::DoubleEndedExactSize,
            IterBox::ExactSize => IterBox::ExactSize,
            IterBox::DoubleEnded | IterBox::Iterator => IterBox::Iterator
        ));

        Ok(this)
    }

    /// Takes two iterators and creates a new iterator over both in sequence.
    ///
    /// `chain()` will return a new iterator which will first iterate over values from the first iterator and then over values from the second iterator.
    ///
    /// In other words, it links two iterators together, in a chain. 🔗
    ///
    /// @param self<T> $other
    /// @return self<T>
    /// @throws \LogicException
    /// @throws \ValueError
    pub fn chain(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        other: ZIterRS,
    ) -> Result<&mut ZendClassObject<ArrayIterator>, IterError> {
        let iter = this.iter.take().ok_or(IterError::Moved)?;
        let mut other_iterator = other.clone();
        let other_iterator = ZendClassObject::<ArrayIterator>::from_zend_obj_mut(
            other_iterator.inner.object_mut().ok_or(IterError::Moved)?,
        )
        .ok_or(IterError::ArgumentError("other".to_string()))?;

        this.iter = Some(match_nested_iter_type!(
            iter,
            other_iterator,
            other_iterator.iter.take().ok_or(anyhow::anyhow!("Iterator is not valid"))?,
            Box::new(iter.chain(other_iterator)),
            IterBox::DoubleEndedExactSize | IterBox::DoubleEnded :
                IterBox::DoubleEndedExactSize | IterBox::DoubleEnded => IterBox::DoubleEnded,
                IterBox::ExactSize | IterBox::Iterator => IterBox::Iterator;
            IterBox::ExactSize | IterBox::Iterator :
                IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator => IterBox::Iterator
        ));

        Ok(this)
    }

    /// ‘Zips up’ two iterators into a single iterator of pairs.
    ///
    /// `zip()` returns a new iterator that will iterate over two other iterators, returning a tuple where the first element comes from the first iterator, and the second element comes from the second iterator.
    ///
    /// In other words, it zips two iterators together, into a single one.
    ///
    /// If either iterator returns `null`, `next()` from the zipped iterator will return `null`. If the zipped iterator has no more elements to return then each further attempt to advance it will first try to advance the first iterator at most one time and if it still yielded an item try to advance the second iterator at most one time.
    ///
    /// To ‘undo’ the result of zipping up two iterators, see `unzip`.
    ///
    /// @param self<U> $other
    /// @return self<array{T, U}>
    /// @throws \LogicException
    /// @throws \ValueError
    pub fn zip(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        other: ZIterRS,
    ) -> Result<&mut ZendClassObject<ArrayIterator>, IterError> {
        let iter = this.iter.take().ok_or(IterError::Moved)?;
        let mut other_iterator = other.clone();
        let other_iterator = ZendClassObject::<ArrayIterator>::from_zend_obj_mut(
            other_iterator.inner.object_mut().ok_or(IterError::Moved)?,
        )
        .ok_or(IterError::ArgumentError("other".to_string()))?;

        this.iter = Some(match_nested_iter_type!(
            iter,
            other_iterator,
            other_iterator.iter.take().ok_or(anyhow::anyhow!("Iterator is not valid"))?,
            Box::new(iter.zip(other_iterator).map(
                |(x, y)| {
                    let mut arr = ZendHashTable::new();
                    arr.push(x.inner);
                    arr.push(y.inner);
                    arr.into_zval(false).unwrap().into()
                },
            )),
            IterBox::DoubleEndedExactSize:
                IterBox::DoubleEndedExactSize => IterBox::DoubleEndedExactSize,
                IterBox::ExactSize => IterBox::ExactSize,
                IterBox::DoubleEnded | IterBox::Iterator => IterBox::Iterator;
            IterBox::ExactSize:
                IterBox::DoubleEndedExactSize | IterBox::ExactSize => IterBox::ExactSize,
                IterBox::DoubleEnded | IterBox::Iterator => IterBox::Iterator;
            IterBox::DoubleEnded | IterBox::Iterator:
                IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator => IterBox::Iterator
        ));

        Ok(this)
    }

    /// Takes a closure and creates an iterator which calls that closure on each element.
    ///
    /// `map()` transforms one iterator into another, by means of its argument. It produces a new iterator which calls this closure on each element of the original iterator.
    ///
    /// If you are good at thinking in types, you can think of `map()` like this: If you have an iterator that gives you elements of some type `A`, and you want an iterator of some other type `B`, you can use `map()`, passing a closure that takes an `A` and returns a `B`.
    ///
    /// `map()` is conceptually similar to a `for` loop. However, as `map()` is lazy, it is best used when you’re already working with other iterators. If you’re doing some sort of looping `for` a side effect, it’s considered more idiomatic to use `for` than `map()`.
    ///
    /// @param callable(T): U $callback
    /// @return self<U>
    /// @throws \LogicException
    #[allow(unreachable_patterns)]
    pub fn map(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        callback: ZCallable,
    ) -> Result<&mut ZendClassObject<ArrayIterator>, IterError> {
        // this.chain.push(Iter::Map { callback });
        let iter = this.iter.take().ok_or(IterError::Moved)?;
        let mut callback = callback;

        this.iter = Some(match_iter_same_type!(
            iter,
            Box::new(iter.map(move |x| { call_cached(&mut callback, [x.inner]).into() })),
            IterBox::DoubleEndedExactSize
                | IterBox::DoubleEnded
                | IterBox::ExactSize
                | IterBox::Iterator
        ));
        Ok(this)
    }

    /// Calls a closure on each element of an iterator.
    ///
    /// This is equivalent to using a `for` loop on the iterator, although break and continue are not possible from a closure. It’s generally more idiomatic to use a `for` loop, but `for_each` may be more legible when processing items at the end of longer iterator chains. In some cases `for_each` may also be faster than a loop, because it will use internal iteration on adapters like `Chain`.
    ///
    /// @param callable(T): void $callback
    /// @throws \LogicException
    pub fn for_each(&mut self, callback: ZCallable) -> Result<(), IterError> {
        let mut callback = callback;
        self.iter.as_mut().map_or(Err(IterError::Consumed), |iter| {
            Ok(match_iter_type!(
                iter,
                iter.for_each(|x| {
                    call_cached(&mut callback, [x.inner]);
                }),
                IterBox::DoubleEndedExactSize
                    | IterBox::DoubleEnded
                    | IterBox::ExactSize
                    | IterBox::Iterator
            ))
        });

        Ok(())
    }

    /// Creates an iterator which uses a closure to determine if an element should be yielded.
    ///
    /// Given an element the closure must return `true` or `false`. The returned iterator will yield only the elements for which the closure returns `true`.
    ///
    /// @param callable(T): bool $callback
    /// @return self<T>
    /// @throws \LogicException
    pub fn filter(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        callback: ZCallable,
    ) -> Result<&mut ZendClassObject<ArrayIterator>, IterError> {
        let iter = this.iter.take().ok_or(IterError::Moved)?;
        let mut callback = callback;
        this.iter = Some(match_iter_result_type!(
            iter,
            Box::new(iter.filter(move |x| {
                call_cached(&mut callback, [x.inner.shallow_clone()])
                    .bool()
                    .unwrap()
            })),
            IterBox::DoubleEndedExactSize | IterBox::DoubleEnded => IterBox::DoubleEnded,
            IterBox::ExactSize | IterBox::Iterator => IterBox::Iterator
        ));

        Ok(this)
    }

    /// Creates an iterator that both filters and maps.
    ///
    /// The returned iterator yields only the values for which the supplied closure returns a
    /// value, discarding null values.
    ///
    /// `filter_map` can be used to make chains of `filter` and `map` more concise.
    ///
    /// @param callable(T): U|null $callback
    /// @return self<U>
    /// @throws \LogicException
    pub fn filter_map(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        callback: ZCallable,
    ) -> Result<&mut ZendClassObject<ArrayIterator>, IterError> {
        let iter = this.iter.take().ok_or(IterError::Moved)?;
        let mut callback = callback;
        this.iter = Some(match_iter_result_type!(
            iter,
            Box::new(
                iter.map(move |x| ZVal::from(call_cached(&mut callback, [x.inner.shallow_clone()])))
                    .filter(|x| !x.inner.is_null())
            ),
            IterBox::DoubleEndedExactSize | IterBox::DoubleEnded => IterBox::DoubleEnded,
            IterBox::ExactSize | IterBox::Iterator => IterBox::Iterator
        ));

        Ok(this)
    }

    /// Creates an iterator which gives the current iteration count as well as the next value.
    ///
    /// The iterator returned yields pairs `[i, val]`, where `i` is the current index of iteration and `val` is the value returned by the iterator.
    ///
    /// `enumerate()` keeps its count as a `usize`. If you want to count by a different sized integer, the `zip` function provides similar functionality.
    ///
    /// # Overflow Behavior
    /// The method does no guarding against overflows, so enumerating more than `usize::MAX` elements either produces the wrong result or panics. If debug assertions are enabled, a panic is guaranteed.
    ///
    /// # Panics
    /// The returned iterator might panic if the to-be-returned index would overflow a `usize`.
    ///
    /// @return self<array{int, T}>
    /// @throws \LogicException
    pub fn enumerate(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
    ) -> Result<&mut ZendClassObject<ArrayIterator>, IterError> {
        let iter = this.iter.take().ok_or(IterError::Moved)?;
        this.iter = Some(match_iter_result_type!(
            iter,
            Box::new(iter.enumerate().map(|(i, x)| {
                let mut arr = ZendHashTable::new();
                arr.push(i);
                arr.push(x.inner);
                arr.into_zval(false).unwrap().into()
            })),
            IterBox::DoubleEndedExactSize => IterBox::DoubleEndedExactSize,
            IterBox::ExactSize => IterBox::ExactSize,
            IterBox::DoubleEnded | IterBox::Iterator => IterBox::Iterator
        ));

        Ok(this)
    }

    // TODO: peekable

    /// Creates an iterator that skips elements based on a predicate.
    ///
    /// `skip_while()` takes a closure as an argument. It will call this closure on each element of the iterator, and ignore elements until it returns `false`.
    ///
    /// After `false` is returned, `skip_while()`’s job is over, and the rest of the elements are yielded.
    ///
    /// @param callable(T): bool $callback
    /// @return self<T>
    /// @throws \LogicException
    pub fn skip_while(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        callback: ZCallable,
    ) -> Result<&mut ZendClassObject<ArrayIterator>, IterError> {
        let iter = this.iter.take().ok_or(IterError::Moved)?;
        let mut callback = callback;
        this.iter = Some(match_iter_result_type!(
            iter,
            Box::new(iter.skip_while(move |x| {
                call_cached(&mut callback, [x.inner.shallow_clone()])
                    .bool()
                    .unwrap()
            })),
            IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator => IterBox::Iterator
        ));

        Ok(this)
    }

    /// Creates an iterator that yields elements based on a predicate.
    ///
    /// `take_while()` takes a closure as an argument. It will call this closure on each element of the iterator, and yield elements while it returns `true`.
    ///
    /// After `false` is returned, `take_while()`’s job is over, and the rest of the elements are ignored.
    ///
    /// @param callable(T): bool $callback
    /// @return self<T>
    /// @throws \LogicException
    pub fn take_while(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        callback: ZCallable,
    ) -> Result<&mut ZendClassObject<ArrayIterator>, IterError> {
        let iter = this.iter.take().ok_or(IterError::Moved)?;
        let mut callback = callback;
        this.iter = Some(match_iter_result_type!(
            iter,
            Box::new(iter.take_while(move |x| {
                call_cached(&mut callback, [x.inner.shallow_clone()])
                    .bool()
                    .unwrap()
            })),
            IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator => IterBox::Iterator
        ));

        Ok(this)
    }

    /// Creates an iterator that both yields elements based on a predicate and maps.
    ///
    /// `map_while()` takes a closure as an argument. It will call this closure on each element of the iterator, and yield elements while it returns a value. Once `null` is returned the rest of the elements are ignored.
    ///
    /// @param callable(T): U|null $callback
    /// @return self<U>
    /// @throws \LogicException
    pub fn map_while(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        callback: ZCallable,
    ) -> Result<&mut ZendClassObject<ArrayIterator>, IterError> {
        let iter = this.iter.take().ok_or(IterError::Moved)?;
        let mut callback = callback;
        this.iter = Some(match_iter_result_type!(
            iter,
            Box::new(
                iter.map(move |x| ZVal::from(call_cached(&mut callback, [x.inner.shallow_clone()])))
                    .take_while(|x| !x.inner.is_null())
            ),
            IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator => IterBox::Iterator
        ));

        Ok(this)
    }

    /// Creates an iterator that skips the first `n` elements.
    ///
    /// `skip(n)` skips elements until `n` elements are skipped or the end of the iterator is reached (whichever happens first). After that, all the remaining elements are yielded. In particular, if the original iterator is too short, then the returned iterator is empty.
    ///
    /// @return self<T>
    /// @throws \LogicException
    pub fn skip(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        n: i64,
    ) -> Result<&mut ZendClassObject<ArrayIterator>, IterError> {
        let iter = this.iter.take().ok_or(IterError::Moved)?;
        this.iter = Some(match_iter_result_type!(
            iter,
            Box::new(iter.skip(n as usize)),
            IterBox::DoubleEndedExactSize => IterBox::DoubleEndedExactSize,
            IterBox::ExactSize => IterBox::ExactSize,
            IterBox::DoubleEnded | IterBox::Iterator => IterBox::Iterator
        ));

        Ok(this)
    }

    /// Creates an iterator that yields the first `n` elements, or fewer if the underlying iterator ends sooner.
    ///
    /// `take(n)` yields elements until `n` elements are yielded or the end of the iterator is reached (whichever happens first). The returned iterator is a prefix of length `n` if the original iterator contains at least `n` elements, otherwise it contains all of the (fewer than `n`) elements of the original iterator.
    ///
    /// @return self<T>
    /// @throws \LogicException
    pub fn take(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        n: i64,
    ) -> Result<&mut ZendClassObject<ArrayIterator>, IterError> {
        let iter = this.iter.take().ok_or(IterError::Moved)?;
        this.iter = Some(match_iter_result_type!(
            iter,
            Box::new(iter.take(n as usize)),
            IterBox::DoubleEndedExactSize => IterBox::DoubleEndedExactSize,
            IterBox::ExactSize => IterBox::ExactSize,
            IterBox::DoubleEnded | IterBox::Iterator => IterBox::Iterator
        ));

        Ok(this)
    }

    // TODO: scan
    // fn scan(
    //     #[this] this: &mut ZendClassObject<ArrayIterator>,
    //     initial: ZVal,
    //     callback: ZCallable,
    // ) -> &mut ZendClassObject<ArrayIterator> {
    //     this.chain.push(Iter::Scan {
    //         initial: initial.inner,
    //         callback,
    //     });
    //     this.double_ended = false;
    //     this.exact_size = false;
    //     this
    // }

    /// Creates an iterator that works like map, but flattens nested structure.
    ///
    /// The `map` adapter is very useful, but only when the closure argument produces values. If it produces an iterator instead, there’s an extra layer of indirection. `flat_map()` will remove this extra layer on its own.
    ///
    /// You can think of `flat_map(f)` as the semantic equivalent of mapping, and then flattening as in `map(f).flatten()`.
    ///
    /// Another way of thinking about `flat_map()`: `map`’s closure returns one item for each element, and `flat_map()`’s closure returns an iterator for each element.
    ///
    /// Note: Currently only supports `array` types.
    ///
    /// @param callable(T): list<U> $callback
    /// @return self<U>
    /// @throws \LogicException
    fn flat_map(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        callback: ZCallable,
    ) -> Result<&mut ZendClassObject<ArrayIterator>, IterError> {
        let iter = this.iter.take().ok_or(IterError::Moved)?;
        let mut callback = callback;
        this.iter = Some(match_iter_result_type!(
            iter,
            Box::new(iter.flat_map(move |x| {
                let arr = call_cached(&mut callback, [x.inner]);
                if let Some(arr) = arr.array() {
                    arr.values().map(|x| x.into()).collect::<Vec<_>>()
                } else {
                    vec![]
                }
            })),
            IterBox::DoubleEndedExactSize | IterBox::DoubleEnded => IterBox::DoubleEnded,
            IterBox::ExactSize | IterBox::Iterator => IterBox::Iterator
        ));

        Ok(this)
    }

    /// Creates an iterator that flattens nested structure.
    ///
    /// This is useful when you have an iterator of iterators or an iterator of things that can be turned into iterators and you want to remove one level of indirection.
    ///
    /// @template U of array<V>
    /// @return self<V>
    /// @throws \LogicException
    fn flatten(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
    ) -> Result<&mut ZendClassObject<ArrayIterator>, IterError> {
        let iter = this.iter.take().ok_or(IterError::Moved)?;
        this.iter = Some(match_iter_result_type!(
            iter,
            Box::new(iter.flat_map(|x| {
                if x.inner.is_array() {
                    let arr = x.inner.array().unwrap();
                    arr.values().map(|x| x.into()).collect::<Vec<_>>()
                } else {
                    vec![x]
                }
            })),
            IterBox::DoubleEndedExactSize | IterBox::DoubleEnded => IterBox::DoubleEnded,
            IterBox::ExactSize | IterBox::Iterator => IterBox::Iterator
        ));

        Ok(this)
    }

    /// This seems to be a no-op in PHP. Keeping it for compatibility.
    ///
    ///
    /// Creates an iterator which ends after the first None.
    ///
    /// After an iterator returns None, future calls may or may not yield Some(T) again. fuse() adapts an iterator, ensuring that after a None is given, it will always return None forever.
    ///
    /// Note that the Fuse wrapper is a no-op on iterators that implement the FusedIterator trait. fuse() may therefore behave incorrectly if the FusedIterator trait is improperly implemented.
    ///
    /// @return self<T>
    /// @throws \LogicException
    fn fuse(
        #[this] this: &mut ZendClassObject<Self>,
    ) -> Result<&mut ZendClassObject<Self>, IterError> {
        let iter = this.iter.take().ok_or(IterError::Moved)?;
        // Not sure how useful this is atm. But keeping it for rust interface compatibility
        this.iter = Some(match_iter_same_type!(
            iter,
            Box::new(iter.fuse()),
            IterBox::DoubleEndedExactSize
                | IterBox::DoubleEnded
                | IterBox::ExactSize
                | IterBox::Iterator
        ));

        Ok(this)
    }

    /// Does something with each element of an iterator, passing the value on.
    ///
    /// When using iterators, you’ll often chain several of them together. While working on such code, you might want to check out what’s happening at various parts in the pipeline. To do that, insert a call to `inspect()`.
    ///
    /// It’s more common for `inspect()` to be used as a debugging tool than to exist in your final code, but applications may find it useful in certain situations when errors need to be logged before being discarded.
    ///
    /// @param callable(T): void $callback
    /// @return self<T>
    /// @throws \LogicException
    fn inspect(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        callback: ZCallable,
    ) -> Result<&mut ZendClassObject<ArrayIterator>, IterError> {
        let iter = this.iter.take().ok_or(IterError::Moved)?;
        let mut callback = callback;
        this.iter = Some(match_iter_same_type!(
            iter,
            Box::new(iter.inspect(move |x| {
                call_cached(&mut callback, [x.inner.shallow_clone()]);
            })),
            IterBox::DoubleEndedExactSize
                | IterBox::DoubleEnded
                | IterBox::ExactSize
                | IterBox::Iterator
        ));

        Ok(this)
    }

    /// Transforms an iterator into an `array`.
    ///
    /// `collect()` can take anything iterable, and turn it into a relevant collection. This is one of the more powerful methods in the standard library, used in a variety of contexts.
    ///
    /// The most basic pattern in which `collect()` is used is to turn one collection into another. You take an iterator do a bunch of transformations, and then `collect()` at the end.
    ///
    /// @return list<T>
    /// @throws \LogicException
    pub fn collect(&mut self) -> Result<Vec<Zval>, IterError> {
        self.iter.as_mut().map_or(Err(IterError::Consumed), |iter| {
            Ok(match_iter_type!(
                iter,
                iter.map(|x| x.inner).collect::<Vec<_>>(),
                IterBox::DoubleEndedExactSize
                    | IterBox::DoubleEnded
                    | IterBox::ExactSize
                    | IterBox::Iterator
            ))
        })
    }

    // TODO: wait for stable
    // pub fn collect_into(&mut self, collection: &mut Zval) -> Result<()> {
    //     let arr: &mut ZendHashTable = collection.array_mut().unwrap();
    //     self.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |iter| {
    //         match_iter_type!(
    //             iter,
    //             for x in iter {
    //                 arr.push(x.inner);
    //             },
    //             IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator
    //         )
    //     })
    // }

    /// Consumes an iterator, creating two collections from it.
    ///
    /// The predicate passed to `partition()` can return `true`, or `false`. `partition()` returns a pair, all of the elements for which it returned true, and all of the elements for which it returned false.
    ///
    /// @param callable(T): bool $callback
    /// @return array{list<T>, list<T>}
    /// @throws \LogicException
    pub fn partition(&mut self, callback: ZCallable) -> Result<ZBox<ZendHashTable>, IterError> {
        let mut callback = callback;
        let (left, right): (Vec<ZVal>, Vec<ZVal>) =
            self.iter
                .as_mut()
                .map_or(Err(IterError::Consumed), |iter| {
                    Ok(match_iter_type!(
                        iter,
                        iter.partition(|x| {
                            call_cached(&mut callback, [x.inner.shallow_clone()])
                                .bool()
                                .unwrap()
                        }),
                        IterBox::DoubleEndedExactSize
                            | IterBox::DoubleEnded
                            | IterBox::ExactSize
                            | IterBox::Iterator
                    ))
                })?; // TODO: check if shallow_clone is necessary

        let mut result = ZendHashTable::new();

        let mut left_result = ZendHashTable::new();
        for x in left {
            left_result.push(x.inner);
        }

        let mut right_result = ZendHashTable::new();
        for x in right {
            right_result.push(x.inner);
        }

        result.push(left_result);
        result.push(right_result);

        Ok(result)
    }

    // TODO: try_fold
    // TODO: try_for_each

    /// Folds every element into an accumulator by applying an operation, returning the final result.
    ///
    /// `fold()` takes two arguments: an initial value, and a closure with two arguments: an ‘accumulator’, and an element. The closure returns the value that the accumulator should have for the next iteration.
    ///
    /// The initial value is the value the accumulator will have on the first call.
    ///
    /// After applying this closure to every element of the iterator, `fold()` returns the accumulator.
    ///
    /// This operation is sometimes called ‘reduce’ or ‘inject’.
    ///
    /// Folding is useful whenever you have a collection of something, and want to produce a single value from it.
    ///
    /// Note: `fold()`, and similar methods that traverse the entire iterator, might not terminate for infinite iterators, even on traits for which a result is determinable in finite time.
    ///
    /// Note: `reduce()` can be used to use the first element as the initial value, if the accumulator type and item type is the same.
    ///
    /// Note: `fold()` combines elements in a left-associative fashion. For associative operators like `+`, the order the elements are combined in is not important, but for non-associative operators like - the order will affect the final result. For a right-associative version of `fold()`, see `rfold()`.
    ///
    /// @param U $initial
    /// @param callable(U, T): U $callback
    /// @return U
    /// @throws \LogicException
    pub fn fold(&mut self, initial: &Zval, callback: ZCallable) -> Result<Zval, IterError> {
        let mut acc = initial.shallow_clone();
        let mut callback = callback;
        self.iter
            .as_mut()
            .map_or(Err(IterError::Consumed), |iter| {
                Ok(match_iter_type!(
                    iter,
                    for x in iter {
                        acc = call_cached(&mut callback, [acc.shallow_clone(), x.inner]);
                    },
                    IterBox::DoubleEndedExactSize
                        | IterBox::DoubleEnded
                        | IterBox::ExactSize
                        | IterBox::Iterator
                ))
            })?; // TODO: Check if correct with shallow_clone

        Ok(acc)
    }

    /// Reduces the elements to a single one, by repeatedly applying a reducing operation.
    ///
    /// If the iterator is empty, returns `null`; otherwise, returns the result of the reduction.
    ///
    /// The reducing function is a closure with two arguments: an ‘accumulator’, and an element. For iterators with at least one element, this is the same as `fold()` with the first element of the iterator as the initial accumulator value, folding every subsequent element into it.
    ///
    /// @param callable(T, T): T $callback
    /// @return T|null
    /// @throws \LogicException
    pub fn reduce(&mut self, callback: ZCallable) -> Result<Option<Zval>, IterError> {
        let mut callback = callback;
        self.iter.as_mut().map_or(Err(IterError::Consumed), |iter| {
            Ok(match_iter_type!(
                iter,
                iter.fold(None, |acc, x| {
                    if let Some(acc) = acc {
                        Some(call_cached(&mut callback, [acc, x.inner]))
                    } else {
                        Some(x.inner)
                    }
                }),
                IterBox::DoubleEndedExactSize
                    | IterBox::DoubleEnded
                    | IterBox::ExactSize
                    | IterBox::Iterator
            ))
        })
    }

    /// Tests if every element of the iterator matches a predicate.
    ///
    /// `all()` takes a closure that returns `true` or `false`. It applies this closure to each element of the iterator, and if they all return `true`, then so does `all()`. If any of them return `false`, it returns `false`.
    ///
    /// `all()` is short-circuiting; in other words, it will stop processing as soon as it finds a `false`, given that no matter what else happens, the result will also be `false`.
    ///
    /// An empty iterator returns `true`.
    ///
    /// @param callable(T): bool $callback
    /// @throws \LogicException
    pub fn all(&mut self, callback: ZCallable) -> Result<bool, IterError> {
        let mut callback = callback;
        self.iter.as_mut().map_or(Err(IterError::Consumed), |iter| {
            Ok(match_iter_type!(
                iter,
                iter.all(|x| { call_cached(&mut callback, [x.inner]).bool().unwrap() }),
                IterBox::DoubleEndedExactSize
                    | IterBox::DoubleEnded
                    | IterBox::ExactSize
                    | IterBox::Iterator
            ))
        })
    }

    /// Tests if any element of the iterator matches a predicate.
    ///
    /// `any()` takes a closure that returns `true` or `false`. It applies this closure to each element of the iterator, and if any of them return `true`, then so does `any()`. If they all return `false`, it returns `false`.
    ///
    /// `any()` is short-circuiting; in other words, it will stop processing as soon as it finds a `true`, given that no matter what else happens, the result will also be `true`.
    ///
    /// An empty iterator returns `false`.
    ///
    /// @param callable(T): bool $callback
    /// @throws \LogicException
    pub fn any(&mut self, callback: ZCallable) -> Result<bool, IterError> {
        let mut callback = callback;
        self.iter.as_mut().map_or(Err(IterError::Consumed), |iter| {
            Ok(match_iter_type!(
                iter,
                iter.any(|x| { call_cached(&mut callback, [x.inner]).bool().unwrap() }),
                IterBox::DoubleEndedExactSize
                    | IterBox::DoubleEnded
                    | IterBox::ExactSize
                    | IterBox::Iterator
            ))
        })
    }

    /// Searches for an element of an iterator that satisfies a predicate.
    ///
    /// `find()` takes a closure that returns `true` or `false`. It applies this closure to each element of the iterator, and if any of them return `true`, then `find()` returns the value. If they all return `false`, it returns `null`.
    ///
    /// `find()` is short-circuiting; in other words, it will stop processing as soon as the closure returns `true`.
    ///
    /// If you need the index of the element, see `position()`.
    ///
    /// @param callable(T): bool $callback
    /// @return T|null
    /// @throws \LogicException
    pub fn find(&mut self, callback: ZCallable) -> Result<Option<Zval>, IterError> {
        let mut callback = callback;
        self.iter.as_mut().map_or(Err(IterError::Consumed), |iter| {
            Ok(match_iter_type!(
                iter,
                iter.find(|x| {
                    call_cached(&mut callback, [x.inner.shallow_clone()])
                        .bool()
                        .unwrap()
                })
                .map(|x| x.inner),
                IterBox::DoubleEndedExactSize
                    | IterBox::DoubleEnded
                    | IterBox::ExactSize
                    | IterBox::Iterator
            ))
        })
    }

    /// Applies function to the elements of iterator and returns the first non-`null` result.
    ///
    /// `iter.find_map(f)` is equivalent to `iter.filter_map(f).next()`.
    ///
    /// @param callable(T): U|null $callback
    /// @return U|null
    /// @throws \LogicException
    pub fn find_map(&mut self, callback: ZCallable) -> Result<Option<Zval>, IterError> {
        let mut callback = callback;
        self.iter.as_mut().map_or(Err(IterError::Consumed), |iter| {
            Ok(match_iter_type!(
                iter,
                iter.find_map(|x| {
                    let res = call_cached(&mut callback, [x.inner]);
                    if res.is_null() {
                        None
                    } else {
                        Some(res)
                    }
                }),
                IterBox::DoubleEndedExactSize
                    | IterBox::DoubleEnded
                    | IterBox::ExactSize
                    | IterBox::Iterator
            ))
        })
    }

    /// Searches for an element in an iterator, returning its index.
    ///
    /// `position()` takes a closure that returns `true` or `false`. It applies this closure to each element of the iterator, and if one of them returns `true`, then `position()` returns the index. If all of them return `false`, it returns `null`.
    ///
    /// `position()` is short-circuiting; in other words, it will stop processing as soon as it finds a `true`.
    ///
    /// # Overflow Behavior
    /// The method does no guarding against overflows, so if there are more than `usize::MAX` non-matching elements, it either produces the wrong result or panics. If debug assertions are enabled, a panic is guaranteed.
    ///
    /// # Panics
    /// This function might panic if the iterator has more than `usize::MAX` non-matching elements.
    ///
    /// @param callable(T): bool $callback
    /// @return int|null
    /// @throws \LogicException
    pub fn position(&mut self, callback: ZCallable) -> Result<Option<i64>, IterError> {
        let mut callback = callback;
        self.iter.as_mut().map_or(Err(IterError::Consumed), |iter| {
            Ok(match_iter_type!(
                iter,
                iter.position(|x| { call_cached(&mut callback, [x.inner]).bool().unwrap() })
                    .map(|x| x as i64),
                IterBox::DoubleEndedExactSize
                    | IterBox::DoubleEnded
                    | IterBox::ExactSize
                    | IterBox::Iterator
            ))
        })
    }

    /// Searches for an element in an iterator from the right, returning its index.
    ///
    /// `rposition()` takes a closure that returns `true` or `false`. It applies this closure to each element of the iterator, starting from the end, and if one of them returns `true`, then rposition() returns the index. If all of them return `false`, it returns `null`.
    ///
    /// `rposition()` is short-circuiting; in other words, it will stop processing as soon as it finds a `true`.
    ///
    /// @param callable(T): bool $callback
    /// @return int|null
    /// @throws \LogicException
    /// @throws \DomainException
    pub fn rposition(&mut self, callback: ZCallable) -> Result<Option<i64>, IterError> {
        self.iter.as_mut().map_or(Err(IterError::Consumed), |iter| {
            let mut callback = callback;
            match_iter_type!(
                iter,
                iter.rposition(|x| { call_cached(&mut callback, [x.inner]).bool().unwrap() })
                    .map(|x| x as i64),
                IterBox::DoubleEndedExactSize,
                _ => Err(IterError::Unsupported("rposition".to_string(), "DoubleEndedExactSize".to_string()))
            )
        })
    }

    /// Returns the maximum element of an iterator.
    ///
    /// If several elements are equally maximum, the last element is returned. If the iterator is empty, `null` is returned.
    ///
    /// @return T|null
    /// @throws \LogicException
    pub fn max(&mut self) -> Result<Option<Zval>, IterError> {
        self.iter.as_mut().map_or(Err(IterError::Consumed), |iter| {
            Ok(match_iter_type!(
                iter,
                iter.max().map(|x| x.inner),
                IterBox::DoubleEndedExactSize
                    | IterBox::DoubleEnded
                    | IterBox::ExactSize
                    | IterBox::Iterator
            ))
        })
    }

    /// Returns the minimum element of an iterator.
    ///
    /// If several elements are equally minimum, the first element is returned. If the iterator is empty, `null` is returned.
    ///
    /// @return T|null
    /// @throws \LogicException
    pub fn min(&mut self) -> Result<Option<Zval>, IterError> {
        self.iter.as_mut().map_or(Err(IterError::Consumed), |iter| {
            Ok(match_iter_type!(
                iter,
                iter.min().map(|x| x.inner),
                IterBox::DoubleEndedExactSize
                    | IterBox::DoubleEnded
                    | IterBox::ExactSize
                    | IterBox::Iterator
            ))
        })
    }

    /// Returns the element that gives the maximum value from the specified function.
    ///
    /// If several elements are equally maximum, the last element is returned. If the iterator is empty, `null` is returned.
    ///
    /// @param callable(T): U $callback
    /// @return T|null
    /// @throws \LogicException
    pub fn max_by_key(&mut self, callback: ZCallable) -> Result<Option<Zval>, IterError> {
        let mut callback = callback;
        self.iter.as_mut().map_or(Err(IterError::Consumed), |iter| {
            Ok(match_iter_type!(
                iter,
                iter.max_by_key(|x| ZVal {
                    inner: call_cached(&mut callback, [x.inner.shallow_clone()])
                })
                .map(|x| x.inner),
                IterBox::DoubleEndedExactSize
                    | IterBox::DoubleEnded
                    | IterBox::ExactSize
                    | IterBox::Iterator
            ))
        })
    }

    /// Returns the element that gives the maximum value with respect to the specified comparison function.
    ///
    /// If several elements are equally maximum, the last element is returned. If the iterator is empty, `null` is returned.
    ///
    /// @param callable(T, T): int $callback
    /// @return T|null
    /// @throws \LogicException
    pub fn max_by(&mut self, callback: ZCallable) -> Result<Option<Zval>, IterError> {
        let mut callback = callback;
        self.iter.as_mut().map_or(Err(IterError::Consumed), |iter| {
            Ok(match_iter_type!(
                iter,
                iter.max_by(|x, y| {
                    call_cached(
                        &mut callback,
                        [x.inner.shallow_clone(), y.inner.shallow_clone()],
                    )
                    .long()
                    .unwrap()
                    .cmp(&0)
                })
                .map(|x| x.inner),
                IterBox::DoubleEndedExactSize
                    | IterBox::DoubleEnded
                    | IterBox::ExactSize
                    | IterBox::Iterator
            ))
        })
    }

    /// Returns the element that gives the minimum value from the specified function.
    ///
    /// If several elements are equally minimum, the first element is returned. If the iterator is empty, `null` is returned.
    ///
    /// @param callable(T): U $callback
    /// @return T|null
    /// @throws \LogicException
    pub fn min_by_key(&mut self, callback: ZCallable) -> Result<Option<Zval>, IterError> {
        let mut callback = callback;
        self.iter.as_mut().map_or(Err(IterError::Consumed), |iter| {
            Ok(match_iter_type!(
                iter,
                iter.min_by_key(|x| ZVal {
                    inner: call_cached(&mut callback, [x.inner.shallow_clone()])
                })
                .map(|x| x.inner),
                IterBox::DoubleEndedExactSize
                    | IterBox::DoubleEnded
                    | IterBox::ExactSize
                    | IterBox::Iterator
            ))
        })
    }

    /// Returns the element that gives the minimum value with respect to the specified comparison function.
    ///
    /// If several elements are equally minimum, the first element is returned. If the iterator is empty, `null` is returned.
    ///
    /// @param callable(T, T): int $callback
    /// @return T|null
    /// @throws \LogicException
    pub fn min_by(&mut self, callback: ZCallable) -> Result<Option<Zval>, IterError> {
        let mut callback = callback;
        self.iter.as_mut().map_or(Err(IterError::Consumed), |iter| {
            Ok(match_iter_type!(
                iter,
                iter.min_by(|x, y| {
                    call_cached(
                        &mut callback,
                        [x.inner.shallow_clone(), y.inner.shallow_clone()],
                    )
                    .long()
                    .unwrap()
                    .cmp(&0)
                })
                .map(|x| x.inner),
                IterBox::DoubleEndedExactSize
                    | IterBox::DoubleEnded
                    | IterBox::ExactSize
                    | IterBox::Iterator
            ))
        })
    }

    /// Reverses an iterator’s direction.
    ///
    /// Usually, iterators iterate from left to right. After using `rev()`, an iterator will instead iterate from right to left.
    ///
    /// This is only possible if the iterator has an end, so `rev()` only works on DoubleEndedIterators.
    ///
    /// @return self<T>
    /// @throws \LogicException
    /// @throws \DomainException
    pub fn rev(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
    ) -> Result<&mut ZendClassObject<ArrayIterator>, IterError> {
        let iter = this.iter.take().ok_or(IterError::Moved)?;
        this.iter = Some(match_iter_same_type!(
            iter,
            Box::new(iter.rev()),
            IterBox::DoubleEndedExactSize | IterBox::DoubleEnded,
            _ => IterError::Unsupported("rev".to_string(), "DoubleEnded".to_string())
        )?);

        Ok(this)
    }

    // TODO: unzip
    // TODO: cycle
    // TODO: sum
    // TODO: product

    /// Lexicographically compares the elements of this Iterator with those of another.
    ///
    /// @param self<T> $other
    /// @throws \LogicException
    pub fn cmp(&mut self, other: &mut ArrayIterator) -> Result<i8, IterError> {
        self.iter
            .as_mut()
            .map_or(Err(IterError::Consumed), |iter| {
                other
                    .iter
                    .as_mut()
                    .map_or(Err(IterError::Consumed), |other| {
                        Ok(match_iter_type!(
                            iter,
                            match_iter_type!(
                                other,
                                iter.cmp(other),
                                IterBox::DoubleEndedExactSize
                                    | IterBox::DoubleEnded
                                    | IterBox::ExactSize
                                    | IterBox::Iterator
                            ),
                            IterBox::DoubleEndedExactSize
                                | IterBox::DoubleEnded
                                | IterBox::ExactSize
                                | IterBox::Iterator
                        ))
                    })
            })
            .map(|x| match x {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            })
    }

    /// Lexicographically compares the PartialOrd elements of this Iterator with those of another. The comparison works like short-circuit evaluation, returning a result without comparing the remaining elements. As soon as an order can be determined, the evaluation stops and a result is returned.
    ///
    /// @param self<T> $other
    /// @return int|null
    /// @throws \LogicException
    pub fn partial_cmp(&mut self, other: &mut ArrayIterator) -> Result<Option<i8>, IterError> {
        self.iter
            .as_mut()
            .map_or(Err(IterError::Consumed), |iter| {
                other
                    .iter
                    .as_mut()
                    .map_or(Err(IterError::Consumed), |other| {
                        Ok(match_iter_type!(
                            iter,
                            match_iter_type!(
                                other,
                                iter.partial_cmp(other),
                                IterBox::DoubleEndedExactSize
                                    | IterBox::DoubleEnded
                                    | IterBox::ExactSize
                                    | IterBox::Iterator
                            ),
                            IterBox::DoubleEndedExactSize
                                | IterBox::DoubleEnded
                                | IterBox::ExactSize
                                | IterBox::Iterator
                        ))
                    })
            })
            .map(|x| match x {
                Some(std::cmp::Ordering::Less) => Some(-1),
                Some(std::cmp::Ordering::Equal) => Some(0),
                Some(std::cmp::Ordering::Greater) => Some(1),
                None => None,
            })
    }

    /// Determines if the elements of this Iterator are equal to those of another.
    ///
    /// @param self<T> $other
    /// @throws \LogicException
    pub fn eq(&mut self, other: &mut ArrayIterator) -> Result<bool, IterError> {
        self.iter.as_mut().map_or(Err(IterError::Consumed), |iter| {
            other
                .iter
                .as_mut()
                .map_or(Err(IterError::Consumed), |other| {
                    Ok(match_iter_type!(
                        iter,
                        match_iter_type!(
                            other,
                            iter.eq(other),
                            IterBox::DoubleEndedExactSize
                                | IterBox::DoubleEnded
                                | IterBox::ExactSize
                                | IterBox::Iterator
                        ),
                        IterBox::DoubleEndedExactSize
                            | IterBox::DoubleEnded
                            | IterBox::ExactSize
                            | IterBox::Iterator
                    ))
                })
        })
    }

    /// Determines if the elements of this Iterator are not equal to those of another.
    ///
    /// @param self<T> $other
    /// @throws \LogicException
    pub fn ne(&mut self, other: &mut ArrayIterator) -> Result<bool, IterError> {
        self.iter.as_mut().map_or(Err(IterError::Consumed), |iter| {
            other
                .iter
                .as_mut()
                .map_or(Err(IterError::Consumed), |other| {
                    Ok(match_iter_type!(
                        iter,
                        match_iter_type!(
                            other,
                            iter.ne(other),
                            IterBox::DoubleEndedExactSize
                                | IterBox::DoubleEnded
                                | IterBox::ExactSize
                                | IterBox::Iterator
                        ),
                        IterBox::DoubleEndedExactSize
                            | IterBox::DoubleEnded
                            | IterBox::ExactSize
                            | IterBox::Iterator
                    ))
                })
        })
    }

    /// Determines if the elements of this Iterator are lexicographically less than those of another.
    ///
    /// @param self<T> $other
    /// @throws \LogicException
    pub fn lt(&mut self, other: &mut ArrayIterator) -> Result<bool, IterError> {
        self.iter.as_mut().map_or(Err(IterError::Consumed), |iter| {
            other
                .iter
                .as_mut()
                .map_or(Err(IterError::Consumed), |other| {
                    Ok(match_iter_type!(
                        iter,
                        match_iter_type!(
                            other,
                            iter.lt(other),
                            IterBox::DoubleEndedExactSize
                                | IterBox::DoubleEnded
                                | IterBox::ExactSize
                                | IterBox::Iterator
                        ),
                        IterBox::DoubleEndedExactSize
                            | IterBox::DoubleEnded
                            | IterBox::ExactSize
                            | IterBox::Iterator
                    ))
                })
        })
    }

    /// Determines if the elements of this Iterator are lexicographically less or equal to those of another.
    ///
    /// @param self<T> $other
    /// @Throws \LogicException
    pub fn le(&mut self, other: &mut ArrayIterator) -> Result<bool, IterError> {
        self.iter.as_mut().map_or(Err(IterError::Consumed), |iter| {
            other
                .iter
                .as_mut()
                .map_or(Err(IterError::Consumed), |other| {
                    Ok(match_iter_type!(
                        iter,
                        match_iter_type!(
                            other,
                            iter.le(other),
                            IterBox::DoubleEndedExactSize
                                | IterBox::DoubleEnded
                                | IterBox::ExactSize
                                | IterBox::Iterator
                        ),
                        IterBox::DoubleEndedExactSize
                            | IterBox::DoubleEnded
                            | IterBox::ExactSize
                            | IterBox::Iterator
                    ))
                })
        })
    }

    /// Determines if the elements of this Iterator are lexicographically greater than those of another.
    ///
    /// @param self<T> $other
    /// @throws \LogicException
    pub fn gt(&mut self, other: &mut ArrayIterator) -> Result<bool, IterError> {
        self.iter.as_mut().map_or(Err(IterError::Consumed), |iter| {
            other
                .iter
                .as_mut()
                .map_or(Err(IterError::Consumed), |other| {
                    Ok(match_iter_type!(
                        iter,
                        match_iter_type!(
                            other,
                            iter.gt(other),
                            IterBox::DoubleEndedExactSize
                                | IterBox::DoubleEnded
                                | IterBox::ExactSize
                                | IterBox::Iterator
                        ),
                        IterBox::DoubleEndedExactSize
                            | IterBox::DoubleEnded
                            | IterBox::ExactSize
                            | IterBox::Iterator
                    ))
                })
        })
    }

    /// Determines if the elements of this Iterator are lexicographically greater than or equal to those of another.
    ///
    /// @param self<T> $other
    /// @throws \LogicException
    pub fn ge(&mut self, other: &mut ArrayIterator) -> Result<bool, IterError> {
        self.iter.as_mut().map_or(Err(IterError::Consumed), |iter| {
            other
                .iter
                .as_mut()
                .map_or(Err(IterError::Consumed), |other| {
                    Ok(match_iter_type!(
                        iter,
                        match_iter_type!(
                            other,
                            iter.ge(other),
                            IterBox::DoubleEndedExactSize
                                | IterBox::DoubleEnded
                                | IterBox::ExactSize
                                | IterBox::Iterator
                        ),
                        IterBox::DoubleEndedExactSize
                            | IterBox::DoubleEnded
                            | IterBox::ExactSize
                            | IterBox::Iterator
                    ))
                })
        })
    }

    // DoubleEndedIterator
    /// Removes and returns an element from the end of the iterator.
    ///
    /// Returns `null` when there are no more elements.
    ///
    /// @return T|null
    /// @throws \LogicException
    /// @throws \DomainException
    pub fn next_back(&mut self) -> Result<Option<Zval>, IterError> {
        self.iter.as_mut().map_or(Err(IterError::Consumed), |iter| {
            match_iter_type!(
                iter,
                iter.next_back().map(|x| x.inner),
                IterBox::DoubleEndedExactSize | IterBox::DoubleEnded,
                _ => Err(IterError::Unsupported("next_back".to_string(), "DoubleEnded".to_string()))
            )
        })
    }

    /// Returns the `n`th element from the end of the iterator.
    ///
    /// This is essentially the reversed version of `nth()`. Although like most indexing operations, the count starts from zero, so `nth_back(0)` returns the first value from the end, `nth_back(1)` the second, and so on.
    ///
    /// Note that all elements between the end and the returned element will be consumed, including the returned element. This also means that calling `nth_back(0)` multiple times on the same iterator will return different elements.
    ///
    /// `nth_back()` will return `null` if `n` is greater than or equal to the length of the iterator.
    ///
    /// @return T|null
    /// @throws \LogicException
    /// @throws \DomainException
    pub fn nth_back(&mut self, n: i64) -> Result<Option<Zval>, IterError> {
        self.iter.as_mut().map_or(Err(IterError::Consumed), |iter| {
            match_iter_type!(
                iter,
                iter.nth_back(n as usize).map(|x| x.inner),
                IterBox::DoubleEndedExactSize | IterBox::DoubleEnded,
                _ => Err(IterError::Unsupported("nth_back".to_string(), "DoubleEnded".to_string()))
            )
        })
    }

    // TODO: try_rfold => needs error as value

    /// An iterator method that reduces the iterator’s elements to a single, final value, starting from the back.
    ///
    /// This is the reverse version of `fold()`: it takes elements starting from the back of the iterator.
    ///
    /// `rfold()` takes two arguments: an initial value, and a closure with two arguments: an ‘accumulator’, and an element. The closure returns the value that the accumulator should have for the next iteration.
    ///
    /// The initial value is the value the accumulator will have on the first call.
    ///
    /// After applying this closure to every element of the iterator, `rfold()` returns the accumulator.
    ///
    /// This operation is sometimes called ‘reduce’ or ‘inject’.
    ///
    /// Folding is useful whenever you have a collection of something, and want to produce a single value from it.
    ///
    /// Note: `rfold()` combines elements in a right-associative fashion. For associative operators like `+`, the order the elements are combined in is not important, but for non-associative operators like `-` the order will affect the final result. For a left-associative version of `rfold()`, see `fold()`.
    ///
    /// @param U $initial
    /// @param callable(U, T): U $callback
    /// @return U
    /// @throws \LogicException
    /// @throws \DomainException
    fn rfold(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        initial: &Zval,
        callback: ZCallable,
    ) -> Result<Zval, IterError> {
        this.iter.as_mut().map_or(Err(IterError::Consumed), |iter| {
            let mut callback = callback;
            let initial = initial.shallow_clone();
            match_iter_type!(
                iter,
                iter.rfold(initial, |acc, x| {
                    call_cached(&mut callback, [acc.shallow_clone(), x.inner])
                }),
                IterBox::DoubleEndedExactSize | IterBox::DoubleEnded,
                _ => Err(IterError::Unsupported("rfold".to_string(), "DoubleEnded".to_string()))
            )
        })
    }

    /// Searches for an element of an iterator from the back that satisfies a predicate.
    ///
    /// `rfind()` takes a closure that returns `true` or `false`. It applies this closure to each element of the iterator, starting at the end, and if any of them return `true`, then `rfind()` returns the element. If they all return `false`, it returns `null`.
    ///
    /// `rfind()` is short-circuiting; in other words, it will stop processing as soon as the closure returns `true`.
    ///
    /// @param callable(T): bool $callback
    /// @return T|null
    /// @throws \LogicException
    /// @throws \DomainException
    fn rfind(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        callback: ZCallable,
    ) -> Result<Option<Zval>, IterError> {
        this.iter.as_mut().map_or(Err(IterError::Consumed), |iter| {
            let mut callback = callback;
            match_iter_type!(
                iter,
                iter.rfind(|x| {
                    call_cached(&mut callback, [x.inner.shallow_clone()]).bool().unwrap()
                }).map(|x| x.inner),
                IterBox::DoubleEndedExactSize | IterBox::DoubleEnded,
                _ => Err(IterError::Unsupported("rfind".to_string(), "DoubleEnded".to_string()))
            )
        })
    }

    // ExactSizeIterator
    /// Returns the exact remaining length of the iterator.
    ///
    /// The implementation ensures that the iterator will return exactly `len()` more times a `T` value, before returning `null`. This method has a default implementation, so you usually should not implement it directly. However, if you can provide a more efficient implementation, you can do so. See the trait-level docs for an example.
    ///
    /// This function has the same safety guarantees as the `size_hint()` function.
    /// @throws \LogicException
    /// @throws \DomainException
    fn len(&mut self) -> Result<i64, IterError> {
        self.iter.as_mut().map_or(Err(IterError::Consumed), |iter| {
            match_iter_type!(
                iter,
                iter.len() as i64,
                IterBox::DoubleEndedExactSize | IterBox::ExactSize,
                _ => Err(IterError::Unsupported("len".to_string(), "ExactSize".to_string()))
            )
        })
    }
}

trait DoubleEndedExactSizeIterator: DoubleEndedIterator + ExactSizeIterator + Iterator {}

impl<T> DoubleEndedExactSizeIterator for T where
    T: DoubleEndedIterator + ExactSizeIterator + Iterator
{
}

enum IterBox<'a> {
    DoubleEnded(Box<dyn DoubleEndedIterator<Item = ZVal> + 'a>),
    DoubleEndedExactSize(Box<dyn DoubleEndedExactSizeIterator<Item = ZVal> + 'a>),
    ExactSize(Box<dyn ExactSizeIterator<Item = ZVal> + 'a>),
    Iterator(Box<dyn Iterator<Item = ZVal> + 'a>),
}

// #[inline(always)]
// fn call_user_func_array<const N: usize>(callback: &Zval, args: [Zval; N]) -> Zval {
//     let mut retval = Zval::new();
//     let _result = unsafe {
//         _call_user_function_impl(
//             std::ptr::null_mut(),
//             callback as *const ffi::_zval_struct as *mut crate::ffi::_zval_struct,
//             &mut retval,
//             N as _,
//             args.as_ptr() as *mut _,
//             std::ptr::null_mut(),
//         )
//     };
//
//     retval
// }

#[inline(always)]
fn call_cached<const N: usize>(callback: &mut ZCallable, args: [Zval; N]) -> Zval {
    let mut retval = Zval::new();
    callback.fci.params = args.as_ptr() as *mut _;
    callback.fci.param_count = N as _;
    callback.fci.retval = &mut retval;
    // println!("callback.fci.param_count: {}", callback.fci.param_count);
    let _result = unsafe {
        zend_call_function(&mut callback.fci, &mut callback.fcc);
    };

    retval
}

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module
}

pub struct ZCallable {
    zval: Zval,
    fci: ffi::zend_fcall_info,
    fcc: ffi::zend_fcall_info_cache,
}

impl<'a> FromZval<'a> for ZCallable {
    const TYPE: DataType = DataType::Callable;

    fn from_zval(zval: &'a Zval) -> Option<Self> {
        let mut fci = ffi::zend_fcall_info {
            size: std::mem::size_of::<ffi::zend_fcall_info>(),
            function_name: ZVal::null().inner,
            retval: std::ptr::null_mut(),
            params: std::ptr::null_mut(),
            object: std::ptr::null_mut(),
            param_count: 0,
            named_params: std::ptr::null_mut(),
        };
        let mut fcc = ffi::zend_fcall_info_cache {
            function_handler: std::ptr::null_mut(),
            calling_scope: std::ptr::null_mut(),
            called_scope: std::ptr::null_mut(),
            object: std::ptr::null_mut(),
            closure: std::ptr::null_mut(),
        };
        unsafe {
            zend_fcall_info_init(
                &mut zval.shallow_clone(),
                0,
                &mut fci,
                &mut fcc,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
        }
        Some(ZCallable {
            zval: zval.shallow_clone(),
            fci,
            fcc,
        })
    }
}

pub struct ZIterRS {
    inner: Zval,
}

impl Clone for ZIterRS {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.shallow_clone(),
        }
    }
}

impl<'a> FromZval<'a> for ZIterRS {
    const TYPE: DataType = DataType::Object(Some("Iter"));

    fn from_zval(zval: &'a Zval) -> Option<Self> {
        Some(ZIterRS {
            inner: zval.shallow_clone(),
        })
    }
}

pub struct ZVal {
    inner: Zval,
}

impl ZVal {
    pub fn null() -> Self {
        let mut inner = Zval::new();
        inner.set_null();
        Self { inner }
    }
}

impl<'a> FromZval<'a> for ZVal {
    const TYPE: DataType = DataType::Mixed;

    fn from_zval(zval: &'a Zval) -> Option<Self> {
        Some(Self {
            inner: zval.shallow_clone(),
        })
    }
}

impl Clone for ZVal {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.shallow_clone(),
        }
    }
}

impl From<&Zval> for ZVal {
    fn from(zval: &Zval) -> Self {
        Self {
            inner: zval.shallow_clone(),
        }
    }
}

impl From<Zval> for ZVal {
    fn from(zval: Zval) -> Self {
        Self { inner: zval }
    }
}

impl Into<Option<Zval>> for ZVal {
    fn into(self) -> Option<zval> {
        if self.inner.is_null() {
            None
        } else {
            Some(self.inner)
        }
    }
}

impl Ord for ZVal {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl PartialOrd for ZVal {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.inner.is_null() && other.inner.is_null() {
            return Some(std::cmp::Ordering::Equal);
        }
        if self.inner.is_bool() && other.inner.is_bool() {
            return Some(self.inner.bool().unwrap().cmp(&other.inner.bool().unwrap()));
        }
        if self.inner.is_double() && other.inner.is_double() {
            return Some(
                self.inner
                    .double()
                    .unwrap()
                    .partial_cmp(&other.inner.double().unwrap())
                    .unwrap(),
            );
        }
        if self.inner.is_long() && other.inner.is_long() {
            return Some(self.inner.long().unwrap().cmp(&other.inner.long().unwrap()));
        }
        if self.inner.is_string() && other.inner.is_string() {
            return Some(
                self.inner
                    .string()
                    .unwrap()
                    .cmp(&other.inner.string().unwrap()),
            );
        }

        if self.inner.is_identical(&other.inner) {
            Some(std::cmp::Ordering::Equal)
        } else {
            None
        }
    }
}

impl PartialEq for ZVal {
    fn eq(&self, other: &Self) -> bool {
        self.inner.is_identical(&other.inner)
    }
}

impl Eq for ZVal {}
