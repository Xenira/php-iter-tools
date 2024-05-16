#![feature(trait_upcasting)]

use std::{borrow::BorrowMut, ptr, usize};

use anyhow::Result;
use ext_php_rs::{
    boxed::ZBox,
    convert::{FromZval, IntoZval},
    ffi::{
        self, _call_user_function_impl, zend_call_function, zend_fcall_info_init,
        zend_hash_get_current_data_ex, zend_hash_internal_pointer_reset_ex,
        zend_hash_move_backwards_ex, zend_hash_move_forward_ex, HashPosition,
    },
    flags::DataType,
    prelude::*,
    types::{ZendClassObject, ZendHashTable, Zval},
};
use macros::{match_iter_result_type, match_iter_same_type, match_nested_iter_type};

use crate::macros::match_iter_type;

mod macros;

#[php_class(name = "Iter")]
pub struct ZDoubleEnded {
    // inner: ZVal,
    inner: Box<dyn Iterator<Item = ZVal> + 'static>,
}

// impl<'a> Into<Box<(dyn Iterator<Item = ZVal> + 'a)>> for IterBuilder {
//     fn into(self) -> Box<(dyn Iterator<Item = ZVal> + 'a)> {
//         let inner = self.inner.inner;
//         let iter = inner.array().unwrap().values().map(|x| ZVal::from(x));
//         let iter = Box::new(iter);
//
//         iter
//     }
// }

pub struct SimpleZValIter<'a> {
    inner: &'a ZendHashTable,
    size: HashPosition,
    pos_front: HashPosition,
    pos_back: HashPosition,
}

impl<'a> Iterator for SimpleZValIter<'a> {
    type Item = ZVal;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos_front == self.pos_back {
            return None;
        }

        let val = unsafe {
            &*zend_hash_get_current_data_ex(
                self.inner as *const ZendHashTable as *mut ZendHashTable,
                &mut self.pos_front as &mut _,
            )
        };

        if ptr::null() == val {
            return None;
        }

        self.pos_front += 1;

        Some(ZVal::from(val))
    }
}

impl<'a> ExactSizeIterator for SimpleZValIter<'a> {
    fn len(&self) -> usize {
        self.size as usize
    }
}

impl<'a> DoubleEndedIterator for SimpleZValIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.pos_back == self.pos_front {
            return None;
        }

        self.pos_back -= 1;
        let val = unsafe {
            &*zend_hash_get_current_data_ex(
                self.inner as *const ZendHashTable as *mut ZendHashTable,
                &mut self.pos_back as &mut _,
            )
        };
        if ptr::null() == val {
            return None;
        }

        Some(ZVal::from(val))
    }
}

#[php_class(name = "ArrayIter")]
pub struct ArrayIterator {
    inner: ZVal,
    chain: Vec<Iter>,
    double_ended: bool,
    exact_size: bool,
    // inner: Box<dyn Iterator<Item = Box<dyn IntoZvalDyn>> + 'static>,
}

impl<'a> IntoIterator for &'a ZVal {
    type Item = ZVal;
    type IntoIter = SimpleZValIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        let arr = self.inner.array().unwrap();
        let pos_front = 0;
        let size = arr.nNumOfElements;
        let pos_back = size;
        return SimpleZValIter {
            inner: arr,
            size,
            pos_front,
            pos_back,
        };
    }
}

#[php_impl]
impl ArrayIterator {
    #[constructor]
    pub fn new(vec: &Zval) -> Self {
        // let inner = vec
        //     .array()
        //     .unwrap()
        //     .values()
        //     .map(|v| v.shallow_clone())
        //     .collect::<Vec<_>>();

        Self {
            inner: ZVal::from(vec),
            chain: Vec::new(),
            double_ended: true,
            exact_size: true,
        }
    }

    pub fn count(&mut self) -> Result<i64> {
        Ok(self.iter()?.count() as i64)
    }

    pub fn last(&mut self) -> Result<Option<Zval>> {
        Ok(self.iter()?.last().map(|x| x.inner))
    }

    pub fn nth(&mut self, n: i64) -> Result<Option<Zval>> {
        Ok(self.iter()?.nth(n as usize).map(|x| x.inner))
    }

    pub fn chain(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        other: ZIterRS,
    ) -> Result<&mut ZendClassObject<ArrayIterator>> {
        let otherIterator = other.clone();
        let otherIterator =
            ZendClassObject::<ArrayIterator>::from_zend_obj(otherIterator.inner.object().unwrap())
                .unwrap();

        this.chain.push(Iter::Chain { other });
        this.exact_size = false;
        this.double_ended = otherIterator.double_ended;
        Ok(this)
    }

    pub fn zip(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        other: ZIterRS,
    ) -> &mut ZendClassObject<ArrayIterator> {
        this.chain.push(Iter::Zip { other });
        this
    }

    pub fn map(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        callback: ZCallable,
    ) -> &mut ZendClassObject<ArrayIterator> {
        this.chain.push(Iter::Map { callback });
        this
    }

    pub fn for_each(&mut self, callback: ZCallable) -> Result<()> {
        self.iter()?.for_each(|x| {
            callback.zval.try_call(vec![&x.inner]).unwrap();
        });

        Ok(())
    }

    pub fn filter(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        callback: ZCallable,
    ) -> &mut ZendClassObject<ArrayIterator> {
        this.chain.push(Iter::Filter { callback });
        this
    }

    pub fn filter_map(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        callback: ZCallable,
    ) -> &mut ZendClassObject<ArrayIterator> {
        this.chain.push(Iter::FilterMap { callback });
        this
    }

    pub fn enumerate(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
    ) -> &mut ZendClassObject<ArrayIterator> {
        this.chain.push(Iter::Enumerate);
        this
    }

    pub fn skip_while(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        callback: ZCallable,
    ) -> &mut ZendClassObject<ArrayIterator> {
        this.chain.push(Iter::SkipWhile { callback });
        this
    }

    pub fn take_while(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        callback: ZCallable,
    ) -> &mut ZendClassObject<ArrayIterator> {
        this.chain.push(Iter::TakeWhile { callback });
        this
    }

    pub fn map_while(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        callback: ZCallable,
    ) -> &mut ZendClassObject<ArrayIterator> {
        this.chain.push(Iter::MapWhile { callback });
        this
    }

    pub fn skip(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        n: i64,
    ) -> &mut ZendClassObject<ArrayIterator> {
        this.chain.push(Iter::Skip(n as usize));
        this
    }

    pub fn take(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        n: i64,
    ) -> &mut ZendClassObject<ArrayIterator> {
        this.chain.push(Iter::Take(n as usize));
        this
    }

    fn flat_map(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        callback: ZCallable,
    ) -> &mut ZendClassObject<ArrayIterator> {
        this.chain.push(Iter::FlatMap { callback });
        this
    }

    fn flatten(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
    ) -> &mut ZendClassObject<ArrayIterator> {
        this.chain.push(Iter::Flatten);
        this
    }

    fn fuse(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
    ) -> &mut ZendClassObject<ArrayIterator> {
        this.chain.push(Iter::Fuse);
        this
    }

    fn inspect(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        callback: ZCallable,
    ) -> &mut ZendClassObject<ArrayIterator> {
        this.chain.push(Iter::Inspect { callback });
        this
    }

    pub fn collect(&mut self) -> Result<Vec<Zval>> {
        Ok(self.iter()?.map(|x| x.inner).collect::<Vec<_>>())
    }

    // TODO: try_collect

    pub fn collect_into(&mut self, collection: &mut Zval) -> Result<()> {
        let arr: &mut ZendHashTable = collection.array_mut().unwrap();
        for x in self.iter()? {
            arr.push(x.inner);
        }

        Ok(())
    }

    pub fn partition(&mut self, callback: ZCallable) -> Result<ZBox<ZendHashTable>> {
        let (left, right): (Vec<ZVal>, Vec<ZVal>) = self.iter()?.partition(|x| {
            callback
                .zval
                .try_call(vec![&x.inner])
                .unwrap()
                .bool()
                .unwrap()
        });

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
        for x in self.iter()? {
            acc = callback.zval.try_call(vec![&acc, &x.inner]).unwrap();
        }

        Ok(acc)
    }

    pub fn reduce(&mut self, callback: ZCallable) -> Result<Option<Zval>> {
        Ok(self.iter()?.fold(None, |acc, x| {
            if let Some(acc) = acc {
                Some(callback.zval.try_call(vec![&acc, &x.inner]).unwrap())
            } else {
                Some(x.inner)
            }
        }))
    }

    // TODO: try_reduce

    pub fn all(&mut self, callback: ZCallable) -> Result<bool> {
        Ok(self.iter()?.all(|x| {
            callback
                .zval
                .try_call(vec![&x.inner])
                .unwrap()
                .bool()
                .unwrap()
        }))
    }

    pub fn any(&mut self, callback: ZCallable) -> Result<bool> {
        Ok(self.iter()?.any(|x| {
            callback
                .zval
                .try_call(vec![&x.inner])
                .unwrap()
                .bool()
                .unwrap()
        }))
    }

    pub fn find(&mut self, callback: ZCallable) -> Result<Option<Zval>> {
        Ok(self
            .iter()?
            .find(|x| {
                callback
                    .zval
                    .try_call(vec![&x.inner])
                    .unwrap()
                    .bool()
                    .unwrap()
            })
            .map(|x| x.inner))
    }

    pub fn find_map(&mut self, callback: ZCallable) -> Result<Option<Zval>> {
        Ok(self.iter()?.find_map(|x| {
            let res = callback.zval.try_call(vec![&x.inner]).unwrap();
            if res.is_null() {
                None
            } else {
                Some(res)
            }
        }))
    }

    pub fn position(&mut self, callback: ZCallable) -> Result<Option<i64>> {
        Ok(self
            .iter()?
            .position(|x| {
                callback
                    .zval
                    .try_call(vec![&x.inner])
                    .unwrap()
                    .bool()
                    .unwrap()
            })
            .map(|x| x as i64))
    }

    // TODO: rposition

    pub fn max(&mut self) -> Result<Option<Zval>> {
        Ok(self.iter()?.max().map(|x| x.inner))
    }

    pub fn min(&mut self) -> Result<Option<Zval>> {
        Ok(self.iter()?.min().map(|x| x.inner))
    }

    pub fn max_by_key(&mut self, callback: ZCallable) -> Result<Option<Zval>> {
        Ok(self
            .iter()?
            .max_by_key(|x| ZVal::from(callback.zval.try_call(vec![&x.inner]).unwrap()))
            .map(|x| x.inner))
    }

    pub fn max_by(&mut self, callback: ZCallable) -> Result<Option<Zval>> {
        Ok(self
            .iter()?
            .max_by(|x, y| {
                callback
                    .zval
                    .try_call(vec![&x.inner, &y.inner])
                    .unwrap()
                    .long()
                    .unwrap()
                    .cmp(&0)
            })
            .map(|x| x.inner))
    }

    pub fn min_by_key(&mut self, callback: ZCallable) -> Result<Option<Zval>> {
        Ok(self
            .iter()?
            .min_by_key(|x| ZVal::from(callback.zval.try_call(vec![&x.inner]).unwrap()))
            .map(|x| x.inner))
    }

    pub fn min_by(&mut self, callback: ZCallable) -> Result<Option<Zval>> {
        Ok(self
            .iter()?
            .min_by(|x, y| {
                callback
                    .zval
                    .try_call(vec![&x.inner, &y.inner])
                    .unwrap()
                    .long()
                    .unwrap()
                    .cmp(&0)
            })
            .map(|x| x.inner))
    }

    // TODO: unzip

    pub fn rev(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
    ) -> Result<&mut ZendClassObject<ArrayIterator>> {
        anyhow::ensure!(this.double_ended, "rev() requires a double-ended iterator");
        this.chain.push(Iter::Rev);

        Ok(this)
    }

    // TODO: sum

    // TODO: product

    pub fn cmp(&mut self, other: &mut ArrayIterator) -> Result<i8> {
        Ok(match self.iter()?.cmp(other.iter()?) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        })
    }

    pub fn partial_cmp(&mut self, other: &mut ArrayIterator) -> Result<Option<i8>> {
        Ok(match self.iter()?.partial_cmp(other.iter()?) {
            Some(std::cmp::Ordering::Less) => Some(-1),
            Some(std::cmp::Ordering::Equal) => Some(0),
            Some(std::cmp::Ordering::Greater) => Some(1),
            None => None,
        })
    }

    pub fn eq(&mut self, other: &mut ArrayIterator) -> Result<bool> {
        Ok(self.iter()?.eq(other.iter()?))
    }

    pub fn ne(&mut self, other: &mut ArrayIterator) -> Result<bool> {
        Ok(self.iter()?.ne(other.iter()?))
    }

    pub fn lt(&mut self, other: &mut ArrayIterator) -> Result<bool> {
        Ok(self.iter()?.lt(other.iter()?))
    }

    pub fn le(&mut self, other: &mut ArrayIterator) -> Result<bool> {
        Ok(self.iter()?.le(other.iter()?))
    }

    pub fn gt(&mut self, other: &mut ArrayIterator) -> Result<bool> {
        Ok(self.iter()?.gt(other.iter()?))
    }

    pub fn ge(&mut self, other: &mut ArrayIterator) -> Result<bool> {
        Ok(self.iter()?.ge(other.iter()?))
    }

    pub fn first(&mut self) -> Result<Option<Zval>> {
        Ok(self.iter()?.next().map(|x| x.inner))
    }
}

impl ArrayIterator {
    fn iter(&mut self) -> Result<Box<dyn Iterator<Item = ZVal> + '_>> {
        match self.iter_box()? {
            IterBox::DoubleEnded(iter) => Ok(iter),
            IterBox::DoubleEndedExactSize(iter) => Ok(iter),
            IterBox::ExactSize(iter) => Ok(iter),
            IterBox::Iterator(iter) => Ok(iter),
        }
    }

    fn iter_box(&mut self) -> Result<IterBox<'_>> {
        Into::<Result<IterBox<'_>>>::into(self)
    }
}

impl<'a> Into<Result<IterBox<'a>>> for &'a mut ArrayIterator {
    #[allow(unreachable_patterns)]
    fn into(self) -> Result<IterBox<'a>> {
        let iter = self.inner.into_iter();
        let mut iter: IterBox<'_> = IterBox::DoubleEndedExactSize(Box::new(iter));

        for chain in self.chain.iter_mut() {
            iter = match chain {
                Iter::Chain { other } => {
                    let other: &mut ZendClassObject<ArrayIterator> =
                        ZendClassObject::<ArrayIterator>::from_zend_obj_mut(
                            other.inner.object_mut().unwrap(),
                        )
                        .unwrap();

                    match_nested_iter_type!(
                        iter,
                        other,
                        other.iter_box()?,
                        Box::new(iter.chain(other)),
                        IterBox::DoubleEndedExactSize | IterBox::DoubleEnded :
                            IterBox::DoubleEndedExactSize | IterBox::DoubleEnded => IterBox::DoubleEnded,
                            IterBox::ExactSize | IterBox::Iterator => IterBox::Iterator;
                        IterBox::ExactSize | IterBox::Iterator :
                            IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator => IterBox::Iterator
                    )?
                }
                Iter::Zip { other } => {
                    let other: &mut ZendClassObject<ArrayIterator> =
                        ZendClassObject::<ArrayIterator>::from_zend_obj_mut(
                            other.inner.object_mut().unwrap(),
                        )
                        .unwrap();

                    match_nested_iter_type!(
                        iter,
                        other,
                        other.iter_box()?,
                        Box::new(iter.zip(other).map(
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
                    )?
                }
                Iter::Map { ref mut callback } => match_iter_same_type!(
                    iter,
                    Box::new(iter.map(move |x| { call_cached(callback, [x.inner]).into() })),
                    IterBox::DoubleEndedExactSize
                        | IterBox::DoubleEnded
                        | IterBox::ExactSize
                        | IterBox::Iterator
                )?,
                Iter::Filter { callback } => match_iter_result_type!(
                    iter,
                    Box::new(iter.filter(move |x| {
                        call_user_func_array(&callback.zval, [x.inner.shallow_clone()])
                            .bool()
                            .unwrap()
                    })),
                    IterBox::DoubleEndedExactSize | IterBox::DoubleEnded => IterBox::DoubleEnded,
                    IterBox::ExactSize | IterBox::Iterator => IterBox::Iterator
                )?,
                Iter::FilterMap { callback } => match_iter_result_type!(
                    iter,
                    Box::new(
                        iter.map(move |x| ZVal::from(call_user_func_array(&callback.zval, [x.inner.shallow_clone()])))
                            .filter(|x| !x.inner.is_null())
                    ),
                    IterBox::DoubleEndedExactSize | IterBox::DoubleEnded => IterBox::DoubleEnded,
                    IterBox::ExactSize | IterBox::Iterator => IterBox::Iterator
                )?,
                Iter::Enumerate => match_iter_result_type!(
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
                )?,
                Iter::SkipWhile { callback } => match_iter_result_type!(
                    iter,
                    Box::new(iter.skip_while(move |x| {
                        call_user_func_array(&callback.zval, [x.inner.shallow_clone()])
                            .bool()
                            .unwrap()
                    })),
                    IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator => IterBox::Iterator
                )?,
                Iter::TakeWhile { callback } => match_iter_result_type!(
                    iter,
                    Box::new(iter.take_while(move |x| {
                        call_user_func_array(&callback.zval, [x.inner.shallow_clone()])
                            .bool()
                            .unwrap()
                    })),
                    IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator => IterBox::Iterator
                )?,
                Iter::MapWhile { callback } => match_iter_result_type!(
                    iter,
                    Box::new(
                        iter.map(move |x| ZVal::from(call_user_func_array(&callback.zval, [x.inner.shallow_clone()])))
                            .take_while(|x| !x.inner.is_null())
                    ),
                    IterBox::DoubleEndedExactSize | IterBox::DoubleEnded | IterBox::ExactSize | IterBox::Iterator => IterBox::Iterator
                )?,
                Iter::Skip(n) => match_iter_result_type!(
                    iter,
                    Box::new(iter.skip(*n)),
                    IterBox::DoubleEndedExactSize => IterBox::DoubleEndedExactSize,
                    IterBox::ExactSize => IterBox::ExactSize,
                    IterBox::DoubleEnded | IterBox::Iterator => IterBox::Iterator
                )?,
                Iter::Take(n) => match_iter_result_type!(
                    iter,
                    Box::new(iter.take(*n)),
                    IterBox::DoubleEndedExactSize => IterBox::DoubleEndedExactSize,
                    IterBox::ExactSize => IterBox::ExactSize,
                    IterBox::DoubleEnded | IterBox::Iterator => IterBox::Iterator
                )?,
                Iter::FlatMap { callback } => match_iter_result_type!(
                    iter,
                    Box::new(iter.flat_map(move |x| {
                        let arr = call_user_func_array(&callback.zval, [x.inner.shallow_clone()]);
                        let arr = arr.array().unwrap();
                        arr.values().map(|x| x.into()).collect::<Vec<_>>()
                    })),
                    IterBox::DoubleEndedExactSize | IterBox::DoubleEnded => IterBox::DoubleEnded,
                    IterBox::ExactSize | IterBox::Iterator => IterBox::Iterator
                )?,
                Iter::Flatten => match_iter_result_type!(
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
                )?,
                Iter::Fuse => match_iter_same_type!(
                    iter,
                    Box::new(
                        iter.map(|x| if x.inner.is_null() { None } else { Some(x) })
                            .fuse()
                            .map(|x| x.unwrap_or(ZVal::null()))
                    ),
                    IterBox::DoubleEndedExactSize
                        | IterBox::DoubleEnded
                        | IterBox::ExactSize
                        | IterBox::Iterator
                )?,
                Iter::Inspect { callback } => match_iter_same_type!(
                    iter,
                    Box::new(iter.inspect(move |x| {
                        call_user_func_array(&callback.zval, [x.inner.shallow_clone()]);
                    })),
                    IterBox::DoubleEndedExactSize
                        | IterBox::DoubleEnded
                        | IterBox::ExactSize
                        | IterBox::Iterator
                )?,
                Iter::Rev => match_iter_same_type!(
                    iter,
                    Box::new(iter.rev()),
                    IterBox::DoubleEndedExactSize | IterBox::DoubleEnded
                )?,
            };
        }

        Ok(iter)
    }
}

enum Iter {
    Chain { other: ZIterRS },
    Zip { other: ZIterRS },
    Map { callback: ZCallable },
    Filter { callback: ZCallable },
    FilterMap { callback: ZCallable },
    Enumerate,
    SkipWhile { callback: ZCallable },
    TakeWhile { callback: ZCallable },
    MapWhile { callback: ZCallable },
    Skip(usize),
    Take(usize),
    // Scan { initial: Zval, callback: ZCallable },
    FlatMap { callback: ZCallable },
    Flatten,
    Fuse,
    Inspect { callback: ZCallable },
    Rev,
    // Cycle,
}

trait DoubleEndedExactSizeIterator: DoubleEndedIterator + ExactSizeIterator {}

impl<T> DoubleEndedExactSizeIterator for T where T: DoubleEndedIterator + ExactSizeIterator {}

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

#[inline(always)]
fn call_user_func_array<const N: usize>(callback: &Zval, args: [Zval; N]) -> Zval {
    let mut retval = Zval::new();
    let _result = unsafe {
        _call_user_function_impl(
            std::ptr::null_mut(),
            callback as *const ffi::_zval_struct as *mut crate::ffi::_zval_struct,
            &mut retval,
            N as _,
            args.as_ptr() as *mut _,
            std::ptr::null_mut(),
        )
    };

    retval
}

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
// struct IterWrapper {
//     inner: Box<dyn Iterator<Item = &'static Zval>>,
// }
//
// impl IterWrapper {rr
//     fn new(inner: Box<dyn Iterator<Item = &'static Zval>>) -> Self {
//         Self { inner }
//     }
//
//     fn map(&self, callback: ZVal) -> Self {
//         let callback = ZendCallable::new_owned(callback.zval).unwrap();
//         Self {
//             inner: Box::new(self.inner.map(move |x| callback.try_call(vec![x]).unwrap())),
//         }
//     }
// }

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

// struct ZVec<T>
// where
//     T: IntoZval,
// {
//     vec: Vec<T>,
// }
//
// impl<'a, T> FromZval<'a> for ZVec<T>
// where
//     T: IntoZval,
// {
//     const TYPE: DataType = DataType::Array;
//
//     fn from_zval(zval: &'a Zval) -> Option<Self> {
//         let mut vec = Vec::new();
//         for val in zval.array().unwrap().values() {
//             vec.push(val)
//         }
//
//         Some(ZVec { vec })
//     }
// }
