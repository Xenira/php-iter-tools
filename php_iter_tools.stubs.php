<?php

// Stubs for php_iter_tools

namespace {
    /**
     * Blazingly 🔥 fast iterator for PHP arrays.
     *
     * @template T
     */
    class Iter {
        /**
         * Creates a new iterator from an array.
         *
         * @param list<T> $vec
         */
        public static function new(mixed $vec): \Iter {}

        /**
         * Advances the iterator and returns the next value.
         *
         * Returns `null` when iteration is finished. Individual iterator implementations may choose to resume iteration, and so calling next() again may or may not eventually start returning values again at some point.
         *
         * @return T|null
         */
        public function next(): mixed {}

        /**
         * Returns the bounds on the remaining length of the iterator.
         *
         * Specifically, size_hint() returns a tuple where the first element is the lower bound, and the second element is the upper bound.
         *
         * The second half of the tuple that is returned is an Option<usize>. A None here means that either there is no known upper bound, or the upper bound is larger than usize.
         *
         * # Implementation notes
         * It is not enforced that an iterator implementation yields the declared number of elements. A buggy iterator may yield less than the lower bound or more than the upper bound of elements.
         *
         * size_hint() is primarily intended to be used for optimizations such as reserving space for the elements of the iterator, but must not be trusted to e.g., omit bounds checks in unsafe code. An incorrect implementation of size_hint() should not lead to memory safety violations.
         *
         * That said, the implementation should provide a correct estimation, because otherwise it would be a violation of the trait’s protocol.
         *
         * The default implementation returns (0, None) which is correct for any iterator.
         *
         * @return array{int, int|null}
         * @throws \LogicException
         * @throws \Exception
         */
        public function sizeHint(): array {}

        /**
         * Consumes the iterator, counting the number of iterations and returning it.
         *
         * This method will call next repeatedly until `null` is encountered, returning the number of times it saw a value. Note that next has to be called at least once even if the iterator does not have any elements.
         *
         * # Overflow Behavior
         * The method does no guarding against overflows, so counting elements of an iterator with more than `usize::MAX` elements either produces the wrong result or panics. If debug assertions are enabled, a panic is guaranteed.
         *
         * # Panics
         * This function might panic if the iterator has more than `usize::MAX` elements.
         */
        public function count(): int {}

        /**
         * Consumes the iterator, returning the last element.
         *
         * This method will evaluate the iterator until it returns `null`. While doing so, it keeps track of the current element. After `null` is returned, `last()` will then return the last element it saw.
         *
         * @return T|null
         */
        public function last(): mixed {}

        /**
         * Returns the nth element of the iterator.
         *
         * Like most indexing operations, the count starts from zero, so `nth(0)` returns the first value, `nth(1)` the second, and so on.
         *
         * Note that all preceding elements, as well as the returned element, will be consumed from the iterator. That means that the preceding elements will be discarded, and also that calling `nth(0)` multiple times on the same iterator will return different elements.
         *
         * `nth()` will return `null` if `n` is greater than or equal to the length of the iterator.
         *
         * @return T|null
         */
        public function nth(int $n): mixed {}

        /**
         * Creates an iterator starting at the same point, but stepping by the given amount at each iteration.
         *
         * Note 1: The first element of the iterator will always be returned, regardless of the step given.
         *
         * Note 2: The time at which ignored elements are pulled is not fixed. StepBy behaves like the sequence `self.next()`, `self.nth(step-1)`, `self.nth(step-1)`, …, but is also free to behave like the sequence `advance_n_and_return_first(&mut self, step)`, `advance_n_and_return_first(&mut self, step)`, … Which way is used may change for some iterators for performance reasons. The second way will advance the iterator earlier and may consume more items.
         *
         * # Panics
         * The method will panic if the given step is 0.
         *
         * @param positive-int $step
         * @return self<T>
         * @throws \LogicException
         * @throws \ValueError
         */
        public function stepBy(int $step): \Iter {}

        /**
         * Takes two iterators and creates a new iterator over both in sequence.
         *
         * `chain()` will return a new iterator which will first iterate over values from the first iterator and then over values from the second iterator.
         *
         * In other words, it links two iterators together, in a chain. 🔗
         *
         * @param self<T> $other
         * @return self<T>
         * @throws \LogicException
         */
        public function chain(\ArrayIter $other): \Iter {}

        /**
         * ‘Zips up’ two iterators into a single iterator of pairs.
         *
         * `zip()` returns a new iterator that will iterate over two other iterators, returning a tuple where the first element comes from the first iterator, and the second element comes from the second iterator.
         *
         * In other words, it zips two iterators together, into a single one.
         *
         * If either iterator returns `null`, `next()` from the zipped iterator will return `null`. If the zipped iterator has no more elements to return then each further attempt to advance it will first try to advance the first iterator at most one time and if it still yielded an item try to advance the second iterator at most one time.
         *
         * To ‘undo’ the result of zipping up two iterators, see `unzip`.
         *
         * @param self<U> $other
         * @return self<array{T, U}>
         * @throws \LogicException
         */
        public function zip(\ArrayIter $other): \Iter {}

        /**
         * Takes a closure and creates an iterator which calls that closure on each element.
         *
         * `map()` transforms one iterator into another, by means of its argument. It produces a new iterator which calls this closure on each element of the original iterator.
         *
         * If you are good at thinking in types, you can think of `map()` like this: If you have an iterator that gives you elements of some type `A`, and you want an iterator of some other type `B`, you can use `map()`, passing a closure that takes an `A` and returns a `B`.
         *
         * `map()` is conceptually similar to a `for` loop. However, as `map()` is lazy, it is best used when you’re already working with other iterators. If you’re doing some sort of looping `for` a side effect, it’s considered more idiomatic to use `for` than `map()`.
         *
         * @param callable(T): U $callback
         * @return self<U>
         * @throws \LogicException
         */
        public function map(callable $callback): \Iter {}

        /**
         * Calls a closure on each element of an iterator.
         *
         * This is equivalent to using a `for` loop on the iterator, although break and continue are not possible from a closure. It’s generally more idiomatic to use a `for` loop, but `for_each` may be more legible when processing items at the end of longer iterator chains. In some cases `for_each` may also be faster than a loop, because it will use internal iteration on adapters like `Chain`.
         *
         * @param callable(T): void $callback
         * @throws \LogicException
         */
        public function forEach(callable $callback): mixed {}

        /**
         * Creates an iterator which uses a closure to determine if an element should be yielded.
         *
         * Given an element the closure must return `true` or `false`. The returned iterator will yield only the elements for which the closure returns `true`.
         *
         * @param callable(T): bool $callback
         * @return self<T>
         * @throws \LogicException
         */
        public function filter(callable $callback): \Iter {}

        /**
         * Creates an iterator that both filters and maps.
         *
         * The returned iterator yields only the values for which the supplied closure returns a
         * value, discarding null values.
         *
         * `filter_map` can be used to make chains of `filter` and `map` more concise.
         *
         * @param callable(T): U|null $callback
         * @return self<U>
         * @throws \LogicException
         */
        public function filterMap(callable $callback): \Iter {}

        /**
         * Creates an iterator which gives the current iteration count as well as the next value.
         *
         * The iterator returned yields pairs `[i, val]`, where `i` is the current index of iteration and `val` is the value returned by the iterator.
         *
         * `enumerate()` keeps its count as a `usize`. If you want to count by a different sized integer, the `zip` function provides similar functionality.
         *
         * # Overflow Behavior
         * The method does no guarding against overflows, so enumerating more than `usize::MAX` elements either produces the wrong result or panics. If debug assertions are enabled, a panic is guaranteed.
         *
         * # Panics
         * The returned iterator might panic if the to-be-returned index would overflow a `usize`.
         *
         * @return self<array{int, T}>
         * @throws \LogicException
         */
        public function enumerate(): \Iter {}

        /**
         * Creates an iterator that skips elements based on a predicate.
         *
         * `skip_while()` takes a closure as an argument. It will call this closure on each element of the iterator, and ignore elements until it returns `false`.
         *
         * After `false` is returned, `skip_while()`’s job is over, and the rest of the elements are yielded.
         *
         * @param callable(T): bool $callback
         * @return self<T>
         * @throws \LogicException
         */
        public function skipWhile(callable $callback): \Iter {}

        /**
         * Creates an iterator that yields elements based on a predicate.
         *
         * `take_while()` takes a closure as an argument. It will call this closure on each element of the iterator, and yield elements while it returns `true`.
         *
         * After `false` is returned, `take_while()`’s job is over, and the rest of the elements are ignored.
         *
         * @param callable(T): bool $callback
         * @return self<T>
         * @throws \LogicException
         */
        public function takeWhile(callable $callback): \Iter {}

        /**
         * Creates an iterator that both yields elements based on a predicate and maps.
         *
         * `map_while()` takes a closure as an argument. It will call this closure on each element of the iterator, and yield elements while it returns a value. Once `null` is returned the rest of the elements are ignored.
         *
         * @param callable(T): U|null $callback
         * @return self<U>
         * @throws \LogicException
         */
        public function mapWhile(callable $callback): \Iter {}

        /**
         * Creates an iterator that skips the first `n` elements.
         *
         * `skip(n)` skips elements until `n` elements are skipped or the end of the iterator is reached (whichever happens first). After that, all the remaining elements are yielded. In particular, if the original iterator is too short, then the returned iterator is empty.
         *
         * @return self<T>
         * @throws \LogicException
         */
        public function skip(int $n): \Iter {}

        /**
         * Creates an iterator that yields the first `n` elements, or fewer if the underlying iterator ends sooner.
         *
         * `take(n)` yields elements until `n` elements are yielded or the end of the iterator is reached (whichever happens first). The returned iterator is a prefix of length `n` if the original iterator contains at least `n` elements, otherwise it contains all of the (fewer than `n`) elements of the original iterator.
         *
         * @return self<T>
         * @throws \LogicException
         */
        public function take(int $n): \Iter {}

        /**
         * Creates an iterator that works like map, but flattens nested structure.
         *
         * The `map` adapter is very useful, but only when the closure argument produces values. If it produces an iterator instead, there’s an extra layer of indirection. `flat_map()` will remove this extra layer on its own.
         *
         * You can think of `flat_map(f)` as the semantic equivalent of mapping, and then flattening as in `map(f).flatten()`.
         *
         * Another way of thinking about `flat_map()`: `map`’s closure returns one item for each element, and `flat_map()`’s closure returns an iterator for each element.
         *
         * Note: Currently only supports `array` types.
         *
         * @param callable(T): list<U> $callback
         * @return self<U>
         * @throws \LogicException
         */
        public function flatMap(callable $callback): \Iter {}

        /**
         * Creates an iterator that flattens nested structure.
         *
         * This is useful when you have an iterator of iterators or an iterator of things that can be turned into iterators and you want to remove one level of indirection.
         *
         * @template U of array<V>
         * @return self<V>
         * @throws \LogicException
         */
        public function flatten(): \Iter {}

        /**
         * This seems to be a no-op in PHP. Keeping it for compatibility.
         *
         *
         * Creates an iterator which ends after the first None.
         *
         * After an iterator returns None, future calls may or may not yield Some(T) again. fuse() adapts an iterator, ensuring that after a None is given, it will always return None forever.
         *
         * Note that the Fuse wrapper is a no-op on iterators that implement the FusedIterator trait. fuse() may therefore behave incorrectly if the FusedIterator trait is improperly implemented.
         *
         * @return self<T>
         * @throws \LogicException
         */
        public function fuse(): \Iter {}

        /**
         * Does something with each element of an iterator, passing the value on.
         *
         * When using iterators, you’ll often chain several of them together. While working on such code, you might want to check out what’s happening at various parts in the pipeline. To do that, insert a call to `inspect()`.
         *
         * It’s more common for `inspect()` to be used as a debugging tool than to exist in your final code, but applications may find it useful in certain situations when errors need to be logged before being discarded.
         *
         * @param callable(T): void $callback
         * @return self<T>
         * @throws \LogicException
         */
        public function inspect(callable $callback): \Iter {}

        /**
         * Transforms an iterator into an `array`.
         *
         * `collect()` can take anything iterable, and turn it into a relevant collection. This is one of the more powerful methods in the standard library, used in a variety of contexts.
         *
         * The most basic pattern in which `collect()` is used is to turn one collection into another. You take an iterator do a bunch of transformations, and then `collect()` at the end.
         *
         * @return list<T>
         * @throws \LogicException
         */
        public function collect(): array {}

        /**
         * Consumes an iterator, creating two collections from it.
         *
         * The predicate passed to `partition()` can return `true`, or `false`. `partition()` returns a pair, all of the elements for which it returned true, and all of the elements for which it returned false.
         *
         * @param callable(T): bool $callback
         * @return array{list<T>, list<T>}
         * @throws \LogicException
         */
        public function partition(callable $callback): array {}

        /**
         * Folds every element into an accumulator by applying an operation, returning the final result.
         *
         * `fold()` takes two arguments: an initial value, and a closure with two arguments: an ‘accumulator’, and an element. The closure returns the value that the accumulator should have for the next iteration.
         *
         * The initial value is the value the accumulator will have on the first call.
         *
         * After applying this closure to every element of the iterator, `fold()` returns the accumulator.
         *
         * This operation is sometimes called ‘reduce’ or ‘inject’.
         *
         * Folding is useful whenever you have a collection of something, and want to produce a single value from it.
         *
         * Note: `fold()`, and similar methods that traverse the entire iterator, might not terminate for infinite iterators, even on traits for which a result is determinable in finite time.
         *
         * Note: `reduce()` can be used to use the first element as the initial value, if the accumulator type and item type is the same.
         *
         * Note: `fold()` combines elements in a left-associative fashion. For associative operators like `+`, the order the elements are combined in is not important, but for non-associative operators like - the order will affect the final result. For a right-associative version of `fold()`, see `rfold()`.
         *
         * @param U $initial
         * @param callable(U, T): U $callback
         * @return U
         * @throws \LogicException
         */
        public function fold(mixed $initial, callable $callback): mixed {}

        /**
         * Reduces the elements to a single one, by repeatedly applying a reducing operation.
         *
         * If the iterator is empty, returns `null`; otherwise, returns the result of the reduction.
         *
         * The reducing function is a closure with two arguments: an ‘accumulator’, and an element. For iterators with at least one element, this is the same as `fold()` with the first element of the iterator as the initial accumulator value, folding every subsequent element into it.
         *
         * @param callable(T, T): T $callback
         * @return T|null
         * @throws \LogicException
         */
        public function reduce(callable $callback): mixed {}

        /**
         * Tests if every element of the iterator matches a predicate.
         *
         * `all()` takes a closure that returns `true` or `false`. It applies this closure to each element of the iterator, and if they all return `true`, then so does `all()`. If any of them return `false`, it returns `false`.
         *
         * `all()` is short-circuiting; in other words, it will stop processing as soon as it finds a `false`, given that no matter what else happens, the result will also be `false`.
         *
         * An empty iterator returns `true`.
         *
         * @param callable(T): bool $callback
         * @throws \LogicException
         */
        public function all(callable $callback): bool {}

        /**
         * Tests if any element of the iterator matches a predicate.
         *
         * `any()` takes a closure that returns `true` or `false`. It applies this closure to each element of the iterator, and if any of them return `true`, then so does `any()`. If they all return `false`, it returns `false`.
         *
         * `any()` is short-circuiting; in other words, it will stop processing as soon as it finds a `true`, given that no matter what else happens, the result will also be `true`.
         *
         * An empty iterator returns `false`.
         *
         * @param callable(T): bool $callback
         * @throws \LogicException
         */
        public function any(callable $callback): bool {}

        /**
         * Searches for an element of an iterator that satisfies a predicate.
         *
         * `find()` takes a closure that returns `true` or `false`. It applies this closure to each element of the iterator, and if any of them return `true`, then `find()` returns the value. If they all return `false`, it returns `null`.
         *
         * `find()` is short-circuiting; in other words, it will stop processing as soon as the closure returns `true`.
         *
         * If you need the index of the element, see `position()`.
         *
         * @param callable(T): bool $callback
         * @return T|null
         * @throws \LogicException
         */
        public function find(callable $callback): mixed {}

        /**
         * Applies function to the elements of iterator and returns the first non-`null` result.
         *
         * `iter.find_map(f)` is equivalent to `iter.filter_map(f).next()`.
         *
         * @param callable(T): U|null $callback
         * @return U|null
         * @throws \LogicException
         */
        public function findMap(callable $callback): mixed {}

        /**
         * Searches for an element in an iterator, returning its index.
         *
         * `position()` takes a closure that returns `true` or `false`. It applies this closure to each element of the iterator, and if one of them returns `true`, then `position()` returns the index. If all of them return `false`, it returns `null`.
         *
         * `position()` is short-circuiting; in other words, it will stop processing as soon as it finds a `true`.
         *
         * # Overflow Behavior
         * The method does no guarding against overflows, so if there are more than `usize::MAX` non-matching elements, it either produces the wrong result or panics. If debug assertions are enabled, a panic is guaranteed.
         *
         * # Panics
         * This function might panic if the iterator has more than `usize::MAX` non-matching elements.
         *
         * @param callable(T): bool $callback
         * @return int|null
         * @throws \LogicException
         */
        public function position(callable $callback): int {}

        /**
         * Searches for an element in an iterator from the right, returning its index.
         *
         * `rposition()` takes a closure that returns `true` or `false`. It applies this closure to each element of the iterator, starting from the end, and if one of them returns `true`, then rposition() returns the index. If all of them return `false`, it returns `null`.
         *
         * `rposition()` is short-circuiting; in other words, it will stop processing as soon as it finds a `true`.
         *
         * @param callable(T): bool $callback
         * @return int|null
         * @throws \LogicException
         * @throws \DomainException
         */
        public function rposition(callable $callback): int {}

        /**
         * Returns the maximum element of an iterator.
         *
         * If several elements are equally maximum, the last element is returned. If the iterator is empty, `null` is returned.
         *
         * @return T|null
         * @throws \LogicException
         */
        public function max(): mixed {}

        /**
         * Returns the minimum element of an iterator.
         *
         * If several elements are equally minimum, the first element is returned. If the iterator is empty, `null` is returned.
         *
         * @return T|null
         * @throws \LogicException
         */
        public function min(): mixed {}

        /**
         * Returns the element that gives the maximum value from the specified function.
         *
         * If several elements are equally maximum, the last element is returned. If the iterator is empty, `null` is returned.
         *
         * @param callable(T): U $callback
         * @return T|null
         * @throws \LogicException
         */
        public function maxByKey(callable $callback): mixed {}

        /**
         * Returns the element that gives the maximum value with respect to the specified comparison function.
         *
         * If several elements are equally maximum, the last element is returned. If the iterator is empty, `null` is returned.
         *
         * @param callable(T, T): int $callback
         * @return T|null
         * @throws \LogicException
         */
        public function maxBy(callable $callback): mixed {}

        /**
         * Returns the element that gives the minimum value from the specified function.
         *
         * If several elements are equally minimum, the first element is returned. If the iterator is empty, `null` is returned.
         *
         * @param callable(T): U $callback
         * @return T|null
         * @throws \LogicException
         */
        public function minByKey(callable $callback): mixed {}

        /**
         * Returns the element that gives the minimum value with respect to the specified comparison function.
         *
         * If several elements are equally minimum, the first element is returned. If the iterator is empty, `null` is returned.
         *
         * @param callable(T, T): int $callback
         * @return T|null
         * @throws \LogicException
         */
        public function minBy(callable $callback): mixed {}

        /**
         * Reverses an iterator’s direction.
         *
         * Usually, iterators iterate from left to right. After using `rev()`, an iterator will instead iterate from right to left.
         *
         * This is only possible if the iterator has an end, so `rev()` only works on DoubleEndedIterators.
         *
         * @return self<T>
         * @throws \LogicException
         * @throws \DomainException
         */
        public function rev(): \Iter {}

        /**
         * Lexicographically compares the elements of this Iterator with those of another.
         *
         * @param self<T> $other
         * @throws \LogicException
         */
        public function cmp(\Iter $other): int {}

        /**
         * Lexicographically compares the PartialOrd elements of this Iterator with those of another. The comparison works like short-circuit evaluation, returning a result without comparing the remaining elements. As soon as an order can be determined, the evaluation stops and a result is returned.
         *
         * @param self<T> $other
         * @return int|null
         * @throws \LogicException
         */
        public function partialCmp(\Iter $other): int {}

        /**
         * Determines if the elements of this Iterator are equal to those of another.
         *
         * @param self<T> $other
         * @throws \LogicException
         */
        public function eq(\Iter $other): bool {}

        /**
         * Determines if the elements of this Iterator are not equal to those of another.
         *
         * @param self<T> $other
         * @throws \LogicException
         */
        public function ne(\Iter $other): bool {}

        /**
         * Determines if the elements of this Iterator are lexicographically less than those of another.
         *
         * @param self<T> $other
         * @throws \LogicException
         */
        public function lt(\Iter $other): bool {}

        /**
         * Determines if the elements of this Iterator are lexicographically less or equal to those of another.
         *
         * @param self<T> $other
         * @Throws \LogicException
         */
        public function le(\Iter $other): bool {}

        /**
         * Determines if the elements of this Iterator are lexicographically greater than those of another.
         *
         * @param self<T> $other
         * @throws \LogicException
         */
        public function gt(\Iter $other): bool {}

        /**
         * Determines if the elements of this Iterator are lexicographically greater than or equal to those of another.
         *
         * @param self<T> $other
         * @throws \LogicException
         */
        public function ge(\Iter $other): bool {}

        /**
         * Removes and returns an element from the end of the iterator.
         *
         * Returns `null` when there are no more elements.
         *
         * @return T|null
         * @throws \LogicException
         * @throws \DomainException
         */
        public function nextBack(): mixed {}

        /**
         * Returns the `n`th element from the end of the iterator.
         *
         * This is essentially the reversed version of `nth()`. Although like most indexing operations, the count starts from zero, so `nth_back(0)` returns the first value from the end, `nth_back(1)` the second, and so on.
         *
         * Note that all elements between the end and the returned element will be consumed, including the returned element. This also means that calling `nth_back(0)` multiple times on the same iterator will return different elements.
         *
         * `nth_back()` will return `null` if `n` is greater than or equal to the length of the iterator.
         *
         * @return T|null
         * @throws \LogicException
         * @throws \DomainException
         */
        public function nthBack(int $n): mixed {}

        /**
         * An iterator method that reduces the iterator’s elements to a single, final value, starting from the back.
         *
         * This is the reverse version of `fold()`: it takes elements starting from the back of the iterator.
         *
         * `rfold()` takes two arguments: an initial value, and a closure with two arguments: an ‘accumulator’, and an element. The closure returns the value that the accumulator should have for the next iteration.
         *
         * The initial value is the value the accumulator will have on the first call.
         *
         * After applying this closure to every element of the iterator, `rfold()` returns the accumulator.
         *
         * This operation is sometimes called ‘reduce’ or ‘inject’.
         *
         * Folding is useful whenever you have a collection of something, and want to produce a single value from it.
         *
         * Note: `rfold()` combines elements in a right-associative fashion. For associative operators like `+`, the order the elements are combined in is not important, but for non-associative operators like `-` the order will affect the final result. For a left-associative version of `rfold()`, see `fold()`.
         *
         * @param U $initial
         * @param callable(U, T): U $callback
         * @return U
         * @throws \LogicException
         * @throws \DomainException
         */
        public function rfold(mixed $initial, callable $callback): mixed {}

        /**
         * Searches for an element of an iterator from the back that satisfies a predicate.
         *
         * `rfind()` takes a closure that returns `true` or `false`. It applies this closure to each element of the iterator, starting at the end, and if any of them return `true`, then `rfind()` returns the element. If they all return `false`, it returns `null`.
         *
         * `rfind()` is short-circuiting; in other words, it will stop processing as soon as the closure returns `true`.
         *
         * @param callable(T): bool $callback
         * @return T|null
         * @throws \LogicException
         * @throws \DomainException
         */
        public function rfind(callable $callback): mixed {}

        /**
         * Returns the exact remaining length of the iterator.
         *
         * The implementation ensures that the iterator will return exactly `len()` more times a `T` value, before returning `null`. This method has a default implementation, so you usually should not implement it directly. However, if you can provide a more efficient implementation, you can do so. See the trait-level docs for an example.
         *
         * This function has the same safety guarantees as the `size_hint()` function.
         * @throws \LogicException
         * @throws \DomainException
         */
        public function len(): int {}
    }
}
