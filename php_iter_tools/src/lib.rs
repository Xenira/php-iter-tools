use ext_php_rs::{
    boxed::ZBox,
    convert::{FromZval, IntoZval},
    flags::DataType,
    prelude::*,
    types::{ZendClassObject, ZendHashTable, Zval},
};

#[php_class(name = "ArrayIter")]
pub struct ArrayIterator {
    inner: Vec<Zval>,
    chain: Vec<Iter>,
    // inner: Box<dyn Iterator<Item = Box<dyn IntoZvalDyn>> + 'static>,
}

#[php_impl]
impl ArrayIterator {
    #[constructor]
    pub fn new(vec: &Zval) -> Self {
        let inner = vec
            .array()
            .unwrap()
            .values()
            .map(|v| v.shallow_clone())
            .collect::<Vec<_>>();

        Self {
            inner,
            chain: Vec::new(),
        }
    }

    pub fn count(&self) -> i64 {
        self.iter().count() as i64
    }

    pub fn last(&self) -> Option<Zval> {
        self.iter().last()
    }

    pub fn nth(&self, n: i64) -> Option<Zval> {
        self.iter().nth(n as usize)
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
            callback.zval.try_call(vec![&x]).unwrap();
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
        self.iter().collect::<Vec<_>>()
    }

    // TODO: try_collect

    pub fn collect_into(&self, collection: &mut Zval) {
        let arr: &mut ZendHashTable = collection.array_mut().unwrap();
        for x in self.iter() {
            arr.push(x);
        }
    }

    pub fn partition(&self, callback: ZCallable) -> ZBox<ZendHashTable> {
        let (left, right): (Vec<Zval>, Vec<Zval>) = self
            .iter()
            .partition(|x| callback.zval.try_call(vec![x]).unwrap().bool().unwrap());

        let mut result = ZendHashTable::new();

        let mut left_result = ZendHashTable::new();
        for x in left {
            left_result.push(x);
        }

        let mut right_result = ZendHashTable::new();
        for x in right {
            right_result.push(x);
        }

        result.push(left_result);
        result.push(right_result);

        result
    }

    // TODO: try_fold

    pub fn fold(&self, initial: &Zval, callback: ZCallable) -> Zval {
        let mut acc = initial.shallow_clone();
        for x in self.iter() {
            acc = callback.zval.try_call(vec![&acc, &x]).unwrap();
        }

        acc
    }

    pub fn reduce(&self, callback: ZCallable) -> Option<Zval> {
        self.iter().fold(None, |acc, x| {
            if let Some(acc) = acc {
                Some(callback.zval.try_call(vec![&acc, &x]).unwrap())
            } else {
                Some(x)
            }
        })
    }

    // TODO: try_reduce

    pub fn all(&self, callback: ZCallable) -> bool {
        self.iter()
            .all(|x| callback.zval.try_call(vec![&x]).unwrap().bool().unwrap())
    }

    pub fn any(&self, callback: ZCallable) -> bool {
        self.iter()
            .any(|x| callback.zval.try_call(vec![&x]).unwrap().bool().unwrap())
    }

    pub fn find(&self, callback: ZCallable) -> Option<Zval> {
        self.iter()
            .find(|x| callback.zval.try_call(vec![x]).unwrap().bool().unwrap())
    }

    pub fn find_map(&self, callback: ZCallable) -> Option<Zval> {
        self.iter().find_map(|x| {
            let res = callback.zval.try_call(vec![&x]).unwrap();
            if res.is_null() {
                None
            } else {
                Some(res)
            }
        })
    }

    pub fn position(&self, callback: ZCallable) -> Option<i64> {
        self.iter()
            .position(|x| callback.zval.try_call(vec![&x]).unwrap().bool().unwrap())
            .map(|x| x as i64)
    }

    // TODO: rposition

    // TODO: max

    // TODO: min

    // TODO: max_by_key

    // TODO: max_by

    // TODO: min_by_key

    // TODO: min_by

    // TODO: rev

    // TODO: unzip

    // TODO: sum

    // TODO: product

    // TODO: cmp

    // TODO: partial_cmp

    // TODO: eq

    // TODO: ne

    // TODO: lt

    // TODO: le

    // TODO: gt

    // TODO: ge

    pub fn first(&self) -> Option<Zval> {
        self.iter().next()
    }
}

impl ArrayIterator {
    pub fn iter(&self) -> Box<(dyn Iterator<Item = Zval> + '_)> {
        let mut iter: Box<dyn Iterator<Item = _>> =
            Box::new(self.inner.iter().map(|x| x.shallow_clone()));
        for chain in &self.chain {
            iter =
                match chain {
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
                            arr.push(x);
                            arr.push(y);
                            arr.into_zval(false).unwrap()
                        }))
                    }
                    Iter::Map { callback } => {
                        Box::new(iter.map(move |x| callback.zval.try_call(vec![&x]).unwrap()))
                    }
                    Iter::Filter { callback } => {
                        Box::new(iter.filter(move |x| {
                            callback.zval.try_call(vec![x]).unwrap().bool().unwrap()
                        }))
                    }
                    Iter::FilterMap { callback } => Box::new(
                        iter.map(|x| callback.zval.try_call(vec![&x]).unwrap())
                            .filter(move |x| !x.is_null()),
                    ),
                    Iter::Enumerate => Box::new(iter.enumerate().map(|(i, x)| {
                        let mut arr = ZendHashTable::new();
                        arr.push(i);
                        arr.push(x);
                        arr.into_zval(false).unwrap()
                    })),
                    Iter::SkipWhile { callback } => Box::new(iter.skip_while(move |x| {
                        callback.zval.try_call(vec![x]).unwrap().bool().unwrap()
                    })),
                    Iter::TakeWhile { callback } => Box::new(iter.take_while(move |x| {
                        callback.zval.try_call(vec![x]).unwrap().bool().unwrap()
                    })),
                    Iter::MapWhile { callback } => Box::new(
                        iter.map(move |x| callback.zval.try_call(vec![&x]).unwrap())
                            .take_while(|x| !x.is_null()),
                    ),
                    Iter::Skip(n) => Box::new(iter.skip(*n)),
                    Iter::Take(n) => Box::new(iter.take(*n)),
                    Iter::FlatMap { callback } => Box::new(iter.flat_map(move |x| {
                        let arr = callback.zval.try_call(vec![&x]).unwrap();
                        let arr = arr.array().unwrap();
                        arr.values().map(|x| x.shallow_clone()).collect::<Vec<_>>()
                    })),
                    Iter::Flatten => Box::new(iter.flat_map(|x| {
                        if x.is_array() {
                            let arr = x.array().unwrap();
                            arr.values().map(|x| x.shallow_clone()).collect::<Vec<_>>()
                        } else {
                            vec![x]
                        }
                    })),
                    Iter::Fuse => return Box::new(iter.take_while(|x| !x.is_null())),
                    Iter::Inspect { callback } => Box::new(iter.inspect(move |x| {
                        callback.zval.try_call(vec![x]).unwrap();
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
