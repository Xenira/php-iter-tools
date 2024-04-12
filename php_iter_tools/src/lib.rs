use ext_php_rs::{
    boxed::ZBox,
    convert::{FromZval, IntoZval},
    ffi::zend_object_compare_t,
    flags::DataType,
    prelude::*,
    types::{ZendClassObject, ZendHashTable, Zval},
};

#[php_class(name = "Iter")]
pub struct IterBuilder {
    inner: ZVal,
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

#[php_class(name = "ArrayIter")]
pub struct ArrayIterator {
    inner: Zval,
    chain: Vec<Iter>,
    // inner: Box<dyn Iterator<Item = Box<dyn IntoZvalDyn>> + 'static>,
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
            inner: vec.shallow_clone(),
            chain: Vec::new(),
        }
    }

    pub fn count(&self) -> i64 {
        self.iter().count() as i64
    }

    pub fn last(&self) -> Option<Zval> {
        self.iter().last().map(|x| x.inner)
    }

    pub fn nth(&self, n: i64) -> Option<Zval> {
        self.iter().nth(n as usize).map(|x| x.inner)
    }

    pub fn chain(
        #[this] this: &mut ZendClassObject<ArrayIterator>,
        other: ZIterRS,
    ) -> &mut ZendClassObject<ArrayIterator> {
        this.chain.push(Iter::Chain { other });
        this
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

    pub fn for_each(&self, callback: ZCallable) {
        self.iter().for_each(|x| {
            callback.zval.try_call(vec![&x.inner]).unwrap();
        });
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

    pub fn collect(&self) -> Vec<Zval> {
        self.iter().map(|x| x.inner).collect::<Vec<_>>()
    }

    // TODO: try_collect

    pub fn collect_into(&self, collection: &mut Zval) {
        let arr: &mut ZendHashTable = collection.array_mut().unwrap();
        for x in self.iter() {
            arr.push(x.inner);
        }
    }

    pub fn partition(&self, callback: ZCallable) -> ZBox<ZendHashTable> {
        let (left, right): (Vec<ZVal>, Vec<ZVal>) = self.iter().partition(|x| {
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

        result
    }

    // TODO: try_fold

    pub fn fold(&self, initial: &Zval, callback: ZCallable) -> Zval {
        let mut acc = initial.shallow_clone();
        for x in self.iter() {
            acc = callback.zval.try_call(vec![&acc, &x.inner]).unwrap();
        }

        acc
    }

    pub fn reduce(&self, callback: ZCallable) -> Option<Zval> {
        self.iter().fold(None, |acc, x| {
            if let Some(acc) = acc {
                Some(callback.zval.try_call(vec![&acc, &x.inner]).unwrap())
            } else {
                Some(x.inner)
            }
        })
    }

    // TODO: try_reduce

    pub fn all(&self, callback: ZCallable) -> bool {
        self.iter().all(|x| {
            callback
                .zval
                .try_call(vec![&x.inner])
                .unwrap()
                .bool()
                .unwrap()
        })
    }

    pub fn any(&self, callback: ZCallable) -> bool {
        self.iter().any(|x| {
            callback
                .zval
                .try_call(vec![&x.inner])
                .unwrap()
                .bool()
                .unwrap()
        })
    }

    pub fn find(&self, callback: ZCallable) -> Option<Zval> {
        self.iter()
            .find(|x| {
                callback
                    .zval
                    .try_call(vec![&x.inner])
                    .unwrap()
                    .bool()
                    .unwrap()
            })
            .map(|x| x.inner)
    }

    pub fn find_map(&self, callback: ZCallable) -> Option<Zval> {
        self.iter().find_map(|x| {
            let res = callback.zval.try_call(vec![&x.inner]).unwrap();
            if res.is_null() {
                None
            } else {
                Some(res)
            }
        })
    }

    pub fn position(&self, callback: ZCallable) -> Option<i64> {
        self.iter()
            .position(|x| {
                callback
                    .zval
                    .try_call(vec![&x.inner])
                    .unwrap()
                    .bool()
                    .unwrap()
            })
            .map(|x| x as i64)
    }

    // TODO: rposition

    pub fn max(&self) -> Option<Zval> {
        self.iter().max().map(|x| x.inner)
    }

    pub fn min(&self) -> Option<Zval> {
        self.iter().min().map(|x| x.inner)
    }

    pub fn max_by_key(&self, callback: ZCallable) -> Option<Zval> {
        self.iter()
            .max_by_key(|x| ZVal::from(callback.zval.try_call(vec![&x.inner]).unwrap()))
            .map(|x| x.inner)
    }

    pub fn max_by(&self, callback: ZCallable) -> Option<Zval> {
        self.iter()
            .max_by(|x, y| {
                callback
                    .zval
                    .try_call(vec![&x.inner, &y.inner])
                    .unwrap()
                    .long()
                    .unwrap()
                    .cmp(&0)
            })
            .map(|x| x.inner)
    }

    pub fn min_by_key(&self, callback: ZCallable) -> Option<Zval> {
        self.iter()
            .min_by_key(|x| ZVal::from(callback.zval.try_call(vec![&x.inner]).unwrap()))
            .map(|x| x.inner)
    }

    pub fn min_by(&self, callback: ZCallable) -> Option<Zval> {
        self.iter()
            .min_by(|x, y| {
                callback
                    .zval
                    .try_call(vec![&x.inner, &y.inner])
                    .unwrap()
                    .long()
                    .unwrap()
                    .cmp(&0)
            })
            .map(|x| x.inner)
    }

    // TODO: rev

    // TODO: unzip

    // TODO: sum

    // TODO: product

    pub fn cmp(&self, other: &ArrayIterator) -> i8 {
        match self.iter().cmp(other.iter()) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        }
    }

    pub fn partial_cmp(&self, other: &ArrayIterator) -> Option<i8> {
        match self.iter().partial_cmp(other.iter()) {
            Some(std::cmp::Ordering::Less) => Some(-1),
            Some(std::cmp::Ordering::Equal) => Some(0),
            Some(std::cmp::Ordering::Greater) => Some(1),
            None => None,
        }
    }

    pub fn eq(&self, other: &ArrayIterator) -> bool {
        self.iter().eq(other.iter())
    }

    pub fn ne(&self, other: &ArrayIterator) -> bool {
        self.iter().ne(other.iter())
    }

    pub fn lt(&self, other: &ArrayIterator) -> bool {
        self.iter().lt(other.iter())
    }

    pub fn le(&self, other: &ArrayIterator) -> bool {
        self.iter().le(other.iter())
    }

    pub fn gt(&self, other: &ArrayIterator) -> bool {
        self.iter().gt(other.iter())
    }

    pub fn ge(&self, other: &ArrayIterator) -> bool {
        self.iter().ge(other.iter())
    }

    pub fn first(&self) -> Option<Zval> {
        self.iter().next().map(|x| x.inner)
    }
}

impl ArrayIterator {
    fn iter(&self) -> Box<dyn Iterator<Item = ZVal> + '_> {
        Into::<Box<dyn Iterator<Item = ZVal>>>::into(self)
    }
}

impl<'a> Into<Box<dyn Iterator<Item = ZVal> + 'a>> for &'a ArrayIterator {
    fn into(self) -> Box<(dyn Iterator<Item = ZVal> + 'a)> {
        let mut iter: Box<dyn Iterator<Item = _>> =
            Box::new(self.inner.array().unwrap().values().map(|x| x.into()));

        for chain in &self.chain {
            iter = match chain {
                Iter::Chain { other } => {
                    let other = ZendClassObject::<ArrayIterator>::from_zend_obj(
                        &other.inner.object().unwrap(),
                    )
                    .unwrap();
                    Box::new(iter.chain(other.iter()))
                }
                Iter::Zip { other } => {
                    let other = ZendClassObject::<ArrayIterator>::from_zend_obj(
                        &other.inner.object().unwrap(),
                    )
                    .unwrap();
                    Box::new(iter.zip(other.iter()).map(|(x, y)| {
                        let mut arr = ZendHashTable::new();
                        arr.push(x.inner);
                        arr.push(y.inner);
                        arr.into_zval(false).unwrap().into()
                    }))
                }
                Iter::Map { callback } => Box::new(
                    iter.map(move |x| callback.zval.try_call(vec![&x.inner]).unwrap().into()),
                ),
                Iter::Filter { callback } => Box::new(iter.filter(move |x| {
                    callback
                        .zval
                        .try_call(vec![&x.inner])
                        .unwrap()
                        .bool()
                        .unwrap()
                })),
                Iter::FilterMap { callback } => Box::new(
                    iter.map(|x| ZVal::from(callback.zval.try_call(vec![&x.inner]).unwrap()))
                        .filter(|x| !x.inner.is_null()),
                ),
                Iter::Enumerate => Box::new(iter.enumerate().map(|(i, x)| {
                    let mut arr = ZendHashTable::new();
                    arr.push(i);
                    arr.push(x.inner);
                    arr.into_zval(false).unwrap().into()
                })),
                Iter::SkipWhile { callback } => Box::new(iter.skip_while(move |x| {
                    callback
                        .zval
                        .try_call(vec![&x.inner])
                        .unwrap()
                        .bool()
                        .unwrap()
                })),
                Iter::TakeWhile { callback } => Box::new(iter.take_while(move |x| {
                    callback
                        .zval
                        .try_call(vec![&x.inner])
                        .unwrap()
                        .bool()
                        .unwrap()
                })),
                Iter::MapWhile { callback } => Box::new(
                    iter.map(move |x| ZVal::from(callback.zval.try_call(vec![&x.inner]).unwrap()))
                        .take_while(|x| !x.inner.is_null()),
                ),
                Iter::Skip(n) => Box::new(iter.skip(*n)),
                Iter::Take(n) => Box::new(iter.take(*n)),
                Iter::FlatMap { callback } => Box::new(iter.flat_map(move |x| {
                    let arr = callback.zval.try_call(vec![&x.inner]).unwrap();
                    let arr = arr.array().unwrap();
                    arr.values().map(|x| x.into()).collect::<Vec<_>>()
                })),
                Iter::Flatten => Box::new(iter.flat_map(|x| {
                    if x.inner.is_array() {
                        let arr = x.inner.array().unwrap();
                        arr.values().map(|x| x.into()).collect::<Vec<_>>()
                    } else {
                        vec![x]
                    }
                })),
                Iter::Fuse => return Box::new(iter.take_while(|x| !x.inner.is_null())),
                Iter::Inspect { callback } => Box::new(iter.inspect(move |x| {
                    callback.zval.try_call(vec![&x.inner]).unwrap();
                })),
            };
        }

        iter
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
    // Rev,
    // Cycle,
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
}

impl<'a> FromZval<'a> for ZCallable {
    const TYPE: DataType = DataType::Callable;

    fn from_zval(zval: &'a Zval) -> Option<Self> {
        Some(ZCallable {
            zval: zval.shallow_clone(),
        })
    }
}

pub struct ZIterRS {
    inner: Zval,
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
