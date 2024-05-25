use std::ptr;

use anyhow::Result;
use ext_php_rs::{
    boxed::ZBox,
    convert::{FromZval, IntoZval},
    ffi::{
        self, _call_user_function_impl, zend_call_function, zend_fcall_info_init,
        zend_hash_get_current_data_ex, zval, HashPosition,
    },
    flags::DataType,
    prelude::*,
    types::{ZendClassObject, ZendHashTable, Zval},
};
use macros::{match_iter_result_type, match_iter_same_type, match_nested_iter_type};

use crate::macros::match_iter_type;

mod macros;

// mod result;

// impl<'a> Into<Box<(dyn Iterator<Item = ZVal> + 'a)>> for IterBuilder {
//     fn into(self) -> Box<(dyn Iterator<Item = ZVal> + 'a)> {
//         let inner = self.inner.inner;
//         let iter = inner.array().unwrap().values().map(|x| ZVal::from(x));
//         let iter = Box::new(iter);
//
//         iter
//     }
// }

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

#[php_class(name = "ArrayIter")]
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
    pub fn new(vec: &Zval) -> Self {
        // let inner = vec
        //     .array()
        //     .unwrap()
        //     .values()
        //     .map(|v| v.shallow_clone())
        //     .collect::<Vec<_>>();

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
    pub fn next(&mut self) -> Result<Option<Zval>> {
        self.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |iter| {
            match_iter_type!(
                iter,
                iter.next().map(|x| x.inner),
                IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator
            )
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
    pub fn size_hint(&mut self) -> Result<ZBox<ZendHashTable>> {
        self.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |iter| {
            let size_hint = match_iter_type!(
                iter,
                iter.size_hint(),
                IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator
            );

            size_hint.map(|(x, y)| {
                let mut map = ZendHashTable::new();
                map.push(x);
                map.push(y.map(|x| {
                    let mut val = Zval::new();
                    val.set_long(x as i64);
                    val
                }).unwrap_or(Zval::new()));

                map
            })
        })
    }

    pub fn count(&mut self) -> Result<i64> {
        self.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |iter| {
            match_iter_type!(
                iter,
                iter.count() as i64,
                IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator
            )
        })
    }

    pub fn last(&mut self) -> Result<Option<Zval>> {
        self.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |iter| {
            match_iter_type!(
                iter,
                iter.last().map(|x| x.inner),
                IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator
            )
        })
    }

    pub fn nth(&mut self, n: i64) -> Result<Option<Zval>> {
        self.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |iter| {
            match_iter_type!(
                iter,
                iter.nth(n as usize).map(|x| x.inner),
                IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator
            )
        })
    }

    pub fn chain(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        other: ZIterRS,
    ) -> Result<&mut ZendClassObject<ArrayIterator>> {
        let iter = this
            .iter
            .take()
            .ok_or(anyhow::anyhow!("Iterator is not valid"))?;
        let mut other_iterator = other.clone();
        let other_iterator = ZendClassObject::<ArrayIterator>::from_zend_obj_mut(
            other_iterator.inner.object_mut().unwrap(),
        )
        .unwrap();

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
        )?);

        Ok(this)
    }

    pub fn zip(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        other: ZIterRS,
    ) -> Result<&mut ZendClassObject<ArrayIterator>> {
        let iter = this
            .iter
            .take()
            .ok_or(anyhow::anyhow!("Iterator is not valid"))
            .unwrap();
        let mut other_iterator = other.clone();
        let other_iterator = ZendClassObject::<ArrayIterator>::from_zend_obj_mut(
            other_iterator.inner.object_mut().unwrap(),
        )
        .unwrap();

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
        )?);

        Ok(this)
    }

    #[allow(unreachable_patterns)]
    pub fn map(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        callback: ZCallable,
    ) -> Result<&mut ZendClassObject<ArrayIterator>> {
        // this.chain.push(Iter::Map { callback });
        let iter = this
            .iter
            .take()
            .ok_or(anyhow::anyhow!("Iterator is not valid"))?;
        let mut callback = callback;

        this.iter = Some(match_iter_same_type!(
            iter,
            Box::new(iter.map(move |x| { call_cached(&mut callback, [x.inner]).into() })),
            IterBox::DoubleEndedExactSize
                | IterBox::DoubleEnded
                | IterBox::ExactSize
                | IterBox::Iterator
        )?);
        Ok(this)
    }

    pub fn for_each(&mut self, callback: ZCallable) -> Result<()> {
        let mut callback = callback;
        self.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |iter| {
            match_iter_type!(
                iter,
                iter.for_each(|x| {
                    call_cached(&mut callback, [x.inner]);
                }),
                IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator
            )
        })?;

        Ok(())
    }

    pub fn filter(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        callback: ZCallable,
    ) -> Result<&mut ZendClassObject<ArrayIterator>> {
        let iter = this.iter.take().ok_or(anyhow::anyhow!(
            "Iterator is not valid. This is most likely because the iterator has already been consumed."
        ))?;
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
        )?);

        Ok(this)
    }

    pub fn filter_map(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        callback: ZCallable,
    ) -> Result<&mut ZendClassObject<ArrayIterator>> {
        let iter = this.iter.take().ok_or(anyhow::anyhow!(
            "Iterator is not valid. This is most likely because the iterator has already been consumed."
        ))?;
        let mut callback = callback;
        this.iter = Some(match_iter_result_type!(
            iter,
            Box::new(
                iter.map(move |x| ZVal::from(call_cached(&mut callback, [x.inner.shallow_clone()])))
                    .filter(|x| !x.inner.is_null())
            ),
            IterBox::DoubleEndedExactSize | IterBox::DoubleEnded => IterBox::DoubleEnded,
            IterBox::ExactSize | IterBox::Iterator => IterBox::Iterator
        )?);

        Ok(this)
    }

    pub fn enumerate(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
    ) -> Result<&mut ZendClassObject<ArrayIterator>> {
        let iter = this.iter.take().ok_or(anyhow::anyhow!(
            "Iterator is not valid. This is most likely because the iterator has already been consumed."
        ))?;
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
        )?);

        Ok(this)
    }

    pub fn skip_while(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        callback: ZCallable,
    ) -> Result<&mut ZendClassObject<ArrayIterator>> {
        let iter = this.iter.take().ok_or(anyhow::anyhow!(
            "Iterator is not valid. This is most likely because the iterator has already been consumed."
        ))?;
        let mut callback = callback;
        this.iter = Some(match_iter_result_type!(
            iter,
            Box::new(iter.skip_while(move |x| {
                call_cached(&mut callback, [x.inner.shallow_clone()])
                    .bool()
                    .unwrap()
            })),
            IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator => IterBox::Iterator
        )?);

        Ok(this)
    }

    pub fn take_while(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        callback: ZCallable,
    ) -> Result<&mut ZendClassObject<ArrayIterator>> {
        let iter = this.iter.take().ok_or(anyhow::anyhow!(
            "Iterator is not valid. This is most likely because the iterator has already been consumed."
        ))?;
        let mut callback = callback;
        this.iter = Some(match_iter_result_type!(
            iter,
            Box::new(iter.take_while(move |x| {
                call_cached(&mut callback, [x.inner.shallow_clone()])
                    .bool()
                    .unwrap()
            })),
            IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator => IterBox::Iterator
        )?);

        Ok(this)
    }

    pub fn map_while(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        callback: ZCallable,
    ) -> Result<&mut ZendClassObject<ArrayIterator>> {
        let iter = this.iter.take().ok_or(anyhow::anyhow!(
            "Iterator is not valid. This is most likely because the iterator has already been consumed."
        ))?;
        let mut callback = callback;
        this.iter = Some(match_iter_result_type!(
            iter,
            Box::new(
                iter.map(move |x| ZVal::from(call_cached(&mut callback, [x.inner.shallow_clone()])))
                    .take_while(|x| !x.inner.is_null())
            ),
            IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator => IterBox::Iterator
        )?);

        Ok(this)
    }

    pub fn skip(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        n: i64,
    ) -> Result<&mut ZendClassObject<ArrayIterator>> {
        let iter = this.iter.take().ok_or(anyhow::anyhow!(
            "Iterator is not valid. This is most likely because the iterator has already been consumed."
        ))?;
        this.iter = Some(match_iter_result_type!(
            iter,
            Box::new(iter.skip(n as usize)),
            IterBox::DoubleEndedExactSize => IterBox::DoubleEndedExactSize,
            IterBox::ExactSize => IterBox::ExactSize,
            IterBox::DoubleEnded | IterBox::Iterator => IterBox::Iterator
        )?);

        Ok(this)
    }

    pub fn take(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        n: i64,
    ) -> Result<&mut ZendClassObject<ArrayIterator>> {
        let iter = this.iter.take().ok_or(anyhow::anyhow!(
            "Iterator is not valid. This is most likely because the iterator has already been consumed."
        ))?;
        this.iter = Some(match_iter_result_type!(
            iter,
            Box::new(iter.take(n as usize)),
            IterBox::DoubleEndedExactSize => IterBox::DoubleEndedExactSize,
            IterBox::ExactSize => IterBox::ExactSize,
            IterBox::DoubleEnded | IterBox::Iterator => IterBox::Iterator
        )?);

        Ok(this)
    }

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

    fn flat_map(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        callback: ZCallable,
    ) -> Result<&mut ZendClassObject<ArrayIterator>> {
        let iter = this.iter.take().ok_or(anyhow::anyhow!(
            "Iterator is not valid. This is most likely because the iterator has already been consumed."
        ))?;
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
        )?);

        Ok(this)
    }

    fn flatten(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
    ) -> Result<&mut ZendClassObject<ArrayIterator>> {
        let iter = this.iter.take().ok_or(anyhow::anyhow!(
            "Iterator is not valid. This is most likely because the iterator has already been consumed."
        ))?;
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
        )?);

        Ok(this)
    }

    fn fuse(#[this] this: &mut ZendClassObject<Self>) -> Result<&mut ZendClassObject<Self>> {
        let iter = this.iter.take().ok_or(anyhow::anyhow!(
            "Iterator is not valid. This is most likely because the iterator has already been consumed."
        ))?;
        // Not sure how useful this is atm. But keeping it for rust interface compatibility
        this.iter = Some(match_iter_same_type!(
            iter,
            Box::new(iter.fuse()),
            IterBox::DoubleEndedExactSize
                | IterBox::DoubleEnded
                | IterBox::ExactSize
                | IterBox::Iterator
        )?);

        Ok(this)
    }

    fn inspect(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        callback: ZCallable,
    ) -> Result<&mut ZendClassObject<ArrayIterator>> {
        let iter = this.iter.take().ok_or(anyhow::anyhow!(
            "Iterator is not valid. This is most likely because the iterator has already been consumed."
        ))?;
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
        )?);

        Ok(this)
    }

    pub fn collect(&mut self) -> Result<Vec<Zval>> {
        self.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |iter| {
            match_iter_type!(
                iter,
                iter.map(|x| x.inner).collect::<Vec<_>>(),
                IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator
            )
        })
    }

    // TODO: try_collect

    pub fn collect_into(&mut self, collection: &mut Zval) -> Result<()> {
        let arr: &mut ZendHashTable = collection.array_mut().unwrap();
        self.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |iter| {
            match_iter_type!(
                iter,
                for x in iter {
                    arr.push(x.inner);
                },
                IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator
            )
        })
    }

    pub fn partition(&mut self, callback: ZCallable) -> Result<ZBox<ZendHashTable>> {
        let mut callback = callback;
        let (left, right): (Vec<ZVal>, Vec<ZVal>) =  self.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |iter| {
            match_iter_type!(
                iter,
                iter.partition(|x| {
                    call_cached(&mut callback, [x.inner.shallow_clone()]).bool().unwrap()
                }),
                IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator
            )
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
    pub fn fold(&mut self, initial: &Zval, callback: ZCallable) -> Result<Zval> {
        let mut acc = initial.shallow_clone();
        let mut callback = callback;
        self.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |iter| {
            match_iter_type!(
                iter,
                for x in iter {
                    acc = call_cached(&mut callback, [acc.shallow_clone(), x.inner]);
                },
                IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator
            )
        })?; // TODO: Check if correct with shallow_clone

        Ok(acc)
    }

    /// Reduces the elements to a single one, by repeatedly applying a reducing operation.
    ///
    /// If the iterator is empty, returns None; otherwise, returns the result of the reduction.
    ///
    /// The reducing function is a closure with two arguments: an ‘accumulator’, and an element. For iterators with at least one element, this is the same as fold() with the first element of the iterator as the initial accumulator value, folding every subsequent element into it.
    pub fn reduce(&mut self, callback: ZCallable) -> Result<Option<Zval>> {
        let mut callback = callback;
        self.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |iter| {
            match_iter_type!(
                iter,
                iter.fold(None, |acc, x| {
                    if let Some(acc) = acc {
                        Some(call_cached(&mut callback, [acc, x.inner]))
                    } else {
                        Some(x.inner)
                    }
                }),
                IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator
            )
        })
    }

    // TODO: try_reduce

    pub fn all(&mut self, callback: ZCallable) -> Result<bool> {
        let mut callback = callback;
        self.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |iter| {
            match_iter_type!(
                iter,
                iter.all(|x| {
                    call_cached(&mut callback, [x.inner]).bool().unwrap()
                }),
                IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator
            )
        })
    }

    pub fn any(&mut self, callback: ZCallable) -> Result<bool> {
        let mut callback = callback;
        self.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |iter| {
            match_iter_type!(
                iter,
                iter.any(|x| {
                    call_cached(&mut callback, [x.inner]).bool().unwrap()
                }),
                IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator
            )
        })
    }

    pub fn find(&mut self, callback: ZCallable) -> Result<Option<Zval>> {
        let mut callback = callback;
        self.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |iter| {
            match_iter_type!(
                iter,
                iter.find(|x| {
                    call_cached(&mut callback, [x.inner.shallow_clone()]).bool().unwrap()
                }).map(|x| x.inner),
                IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator
            )
        })
    }

    pub fn find_map(&mut self, callback: ZCallable) -> Result<Option<Zval>> {
        let mut callback = callback;
        self.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |iter| {
            match_iter_type!(
                iter,
                iter.find_map(|x| {
                    let res = call_cached(&mut callback, [x.inner]);
                    if res.is_null() {
                        None
                    } else {
                        Some(res)
                    }
                }),
                IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator
            )
        })
    }

    pub fn position(&mut self, callback: ZCallable) -> Result<Option<i64>> {
        let mut callback = callback;
        self.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |iter| {
            match_iter_type!(
                iter,
                iter.position(|x| {
                    call_cached(&mut callback, [x.inner]).bool().unwrap()
                }).map(|x| x as i64),
                IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator
            )
        })
    }

    // TODO: rposition

    pub fn max(&mut self) -> Result<Option<Zval>> {
        self.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |iter| {
            match_iter_type!(
                iter,
                iter.max().map(|x| x.inner),
                IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator
            )
        })
    }

    pub fn min(&mut self) -> Result<Option<Zval>> {
        self.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |iter| {
            match_iter_type!(
                iter,
                iter.min().map(|x| x.inner),
                IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator
            )
        })
    }

    pub fn max_by_key(&mut self, callback: ZCallable) -> Result<Option<Zval>> {
        let mut callback = callback;
        self.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |iter| {
            match_iter_type!(
                iter,
                iter.max_by_key(|x| ZVal { inner: call_cached(&mut callback, [x.inner.shallow_clone()]) } ).map(|x| x.inner),
                IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator
            )
        })
    }

    pub fn max_by(&mut self, callback: ZCallable) -> Result<Option<Zval>> {
        let mut callback = callback;
        self.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |iter| {
            match_iter_type!(
                iter,
                iter.max_by(|x, y| {
                    call_cached(&mut callback, [x.inner.shallow_clone(), y.inner.shallow_clone()]).long().unwrap().cmp(&0)
                }).map(|x| x.inner),
                IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator
            )
        })
    }

    pub fn min_by_key(&mut self, callback: ZCallable) -> Result<Option<Zval>> {
        let mut callback = callback;
        self.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |iter| {
            match_iter_type!(
                iter,
                iter.min_by_key(|x| ZVal { inner: call_cached(&mut callback, [x.inner.shallow_clone()]) } ).map(|x| x.inner),
                IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator
            )
        })
    }

    pub fn min_by(&mut self, callback: ZCallable) -> Result<Option<Zval>> {
        let mut callback = callback;
        self.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |iter| {
            match_iter_type!(
                iter,
                iter.min_by(|x, y| {
                    call_cached(&mut callback, [x.inner.shallow_clone(), y.inner.shallow_clone()]).long().unwrap().cmp(&0)
                }).map(|x| x.inner),
                IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator
            )
        })
    }

    // TODO: unzip

    pub fn rev(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
    ) -> Result<&mut ZendClassObject<ArrayIterator>> {
        let iter = this.iter.take().ok_or(anyhow::anyhow!(
            "Iterator is not valid. This is most likely because the iterator has already been consumed."
        ))?;
        this.iter = Some(match_iter_same_type!(
            iter,
            Box::new(iter.rev()),
            IterBox::DoubleEndedExactSize | IterBox::DoubleEnded
        )?);

        Ok(this)
    }

    // TODO: sum

    // TODO: product

    pub fn cmp(&mut self, other: &mut ArrayIterator) -> Result<i8> {
        self.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |iter| {
            other.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |other| {
                match_iter_type!(
                    iter,
                    match_iter_type!(
                        other,
                        iter.cmp(other),
                        IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator
                    ),
                    IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator
                )?
            })
        }).map(|x| match x {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        })
    }

    pub fn partial_cmp(&mut self, other: &mut ArrayIterator) -> Result<Option<i8>> {
        self.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |iter| {
            other.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |other| {
                match_iter_type!(
                    iter,
                    match_iter_type!(
                        other,
                        iter.partial_cmp(other),
                        IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator
                    ),
                    IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator
                )?
            })
        }).map(|x| match x {
            Some(std::cmp::Ordering::Less) => Some(-1),
            Some(std::cmp::Ordering::Equal) => Some(0),
            Some(std::cmp::Ordering::Greater) => Some(1),
            None => None,
        })
    }

    pub fn eq(&mut self, other: &mut ArrayIterator) -> Result<bool> {
        self.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |iter| {
            other.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |other| {
                match_iter_type!(
                    iter,
                    match_iter_type!(
                        other,
                        iter.eq(other),
                        IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator
                    ),
                    IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator
                )?
            })
        })
    }

    pub fn ne(&mut self, other: &mut ArrayIterator) -> Result<bool> {
        self.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |iter| {
            other.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |other| {
                match_iter_type!(
                    iter,
                    match_iter_type!(
                        other,
                        iter.ne(other),
                        IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator
                    ),
                    IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator
                )?
            })
        })
    }

    pub fn lt(&mut self, other: &mut ArrayIterator) -> Result<bool> {
        self.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |iter| {
            other.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |other| {
                match_iter_type!(
                    iter,
                    match_iter_type!(
                        other,
                        iter.lt(other),
                        IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator
                    ),
                    IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator
                )?
            })
        })
    }

    pub fn le(&mut self, other: &mut ArrayIterator) -> Result<bool> {
        self.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |iter| {
            other.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |other| {
                match_iter_type!(
                    iter,
                    match_iter_type!(
                        other,
                        iter.le(other),
                        IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator
                    ),
                    IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator
                )?
            })
        })
    }

    pub fn gt(&mut self, other: &mut ArrayIterator) -> Result<bool> {
        self.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |iter| {
            other.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |other| {
                match_iter_type!(
                    iter,
                    match_iter_type!(
                        other,
                        iter.gt(other),
                        IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator
                    ),
                    IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator
                )?
            })
        })
    }

    pub fn ge(&mut self, other: &mut ArrayIterator) -> Result<bool> {
        self.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |iter| {
            other.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |other| {
                match_iter_type!(
                    iter,
                    match_iter_type!(
                        other,
                        iter.ge(other),
                        IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator
                    ),
                    IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator
                )?
            })
        })
    }

    // DoubleEndedIterator
    pub fn next_back(&mut self) -> Result<Option<Zval>> {
        self.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |iter| {
            match_iter_type!(
                iter,
                iter.next_back().map(|x| x.inner),
                IterBox::DoubleEndedExactSize | IterBox::DoubleEnded
            )
        })
    }

    pub fn nth_back(&mut self, n: i64) -> Result<Option<Zval>> {
        self.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |iter| {
            match_iter_type!(
                iter,
                iter.nth_back(n as usize).map(|x| x.inner),
                IterBox::DoubleEndedExactSize | IterBox::DoubleEnded
            )
        })
    }

    // try_rfold => needs error as value

    fn rfold(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        initial: &Zval,
        callback: ZCallable,
    ) -> Result<Zval> {
        this.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |iter| {
        let mut callback = callback;
        let initial = initial.shallow_clone();
            match_iter_type!(
                iter,
                iter.rfold(initial, |acc, x| {
                    call_cached(&mut callback, [acc.shallow_clone(), x.inner])
                }),
                IterBox::DoubleEndedExactSize | IterBox::DoubleEnded
            )
        })
    }

    fn rfind(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        callback: ZCallable,
    ) -> Result<Option<Zval>> {
        this.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |iter| {
            let mut callback = callback;
            match_iter_type!(
                iter,
                iter.rfind(|x| {
                    call_cached(&mut callback, [x.inner.shallow_clone()]).bool().unwrap()
                }).map(|x| x.inner),
                IterBox::DoubleEndedExactSize | IterBox::DoubleEnded
            )
        })
    }

    // ExactSizeIterator
    fn len(&mut self) -> Result<i64> {
        self.iter.as_mut().map_or(Err(anyhow::anyhow!("Iterator is not valid. This is most likely because the iterator has already been consumed.")), |iter| {
            match_iter_type!(
                iter,
                iter.len() as i64,
                IterBox::DoubleEndedExactSize | IterBox::ExactSize
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

#[php_function]
pub fn map_array(array: &ZendHashTable, mut callback: ZCallable) -> Vec<Zval> {
    // let callback = ZendCallable::new_owned(callback.shallow_clone()).unwrap();
    let mut result = ZendHashTable::new();
    // for a in array.iter().map(|(_, x)| x.long().unwrap() * 2i64) {
    // let res = call_user_func!(callback, x.shallow_clone());
    // result.push(res);
    // }
    (0..10_000)
        // array
        // .iter()
        // .map(|(_, x)| x.long().unwrap() * 2i64)
        // .collect::<Vec<_>>()
        .map(|x| {
            let mut val = Zval::new();
            val.set_long(x);
            return call_cached(&mut callback, [val]);
        })
        // .map(|(_, x)| call_cached(&mut callback, [x.shallow_clone()]))
        .collect()
    // .for_each(|x| {
    // result.push(x * 2i64);
    // });

    // result

    // array
    //     .iter()
    //     .map(|(_, x)| callback.try_call(vec![x]).unwrap())
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
    const TYPE: DataType = DataType::Object(Some("ArrayIter"));

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
