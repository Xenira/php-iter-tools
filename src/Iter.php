<?php

namespace Xenira\IterTools;

use Iterator;
use Xenira\IterTools\Iter\Chain;
use Xenira\IterTools\Iter\Cycle;
use Xenira\IterTools\Iter\Enumerator;
use Xenira\IterTools\Iter\Filter;
use Xenira\IterTools\Iter\Flatten;
use Xenira\IterTools\Iter\Interleave;
use Xenira\IterTools\Iter\Map;
use Xenira\IterTools\Iter\Take;
use Xenira\IterTools\Iter\TakeWhile;
use Xenira\IterTools\Iter\Zip;

/**
 * @template   T
 * @implements Iterator<T>
 */
abstract class Iter implements Iterator
{
    protected int $position = 0;

    /**
     * @param Iterator<T> $iterator
     */
    protected function __construct(private Iterator $iterator)
    {
    }

    /**
     * @param  callable(T): bool $callback
     * @return Filter<T>
     */
    public function filter(callable $callback): Filter
    {
        return new Filter($this, $callback);
    }
    /**
     * @template U
     * @param    callable(T): U $callback
     * @return   Map<T, U>
     */
    public function map(callable $callback): Map
    {
        return new Map($this, $callback);
    }
    /**
     * @template U
     * @param    callable(U, T): U $callback
     * @param    U                 $initial
     */
    public function reduce(callable $callback, $initial): mixed
    {
        while ($this->valid()) {
            $current = $this->current();
            assert($current !== null);

            $initial = $callback($initial, $current);
            $this->next();
        }
        return $initial;
    }

    /**
     * @return Take<T>
     */
    public function take(int $n): Take
    {
        return new Take($this, $n);
    }

    /**
     * @param  callable(T): bool $callback
     * @return TakeWhile<T>
     */
    public function takeWhile(callable $callback): TakeWhile
    {
        return new TakeWhile($this, $callback);
    }

    /**
     * @return Iter<T>
     */
    public function skip(int $n): Iter
    {
        $this->validateSkip($n);

        while ($n > 0) {
            $this->next();
            $n--;
        }

        return $this;
    }

    protected function validateSkip(int $n): void
    {
        if ($n < 0) {
            throw new \InvalidArgumentException("n must be greater than or equal to 0");
        }
    }

    /**
     * @param  callable(T): bool $callback
     * @return Iter<T>
     */
    public function skipWhile(callable $callback): Iter
    {
        while ($this->valid()) {
            $current = $this->current();
            assert($current !== null);

            if (!$callback($current)) {
                break;
            }
            $this->skip(1);
        }
        return $this;
    }

    /**
     * @param  int $start inclusive
     * @param  int $end   exclusive
     * @return Iter<T>
     */
    public function slice(int $start, int $end): Iter
    {
        if ($start < 0) {
            throw new \InvalidArgumentException("start must be greater than or equal to 0");
        }
        if ($end < 0) {
            throw new \InvalidArgumentException("end must be greater than or equal to 0");
        }
        if ($end < $start) {
            throw new \InvalidArgumentException("end must be greater than or equal to start");
        }

        return $this->skip($start)->take($end - $start);
    }

    /**
     * @return T[]
     */
    public function collect(): array
    {
        $result = [];
        while ($this->valid()) {
            $current = $this->current();
            assert($current !== null);

            $result[] = $current;
            $this->next();
        }
        return $result;
    }

    /**
     * @param  callable(T): bool $callback
     * @return int
     */
    public function count(?callable $callback = null): int
    {
        $iterator = $this;
        if ($callback !== null) {
            $iterator = $this->filter($callback);
        }

        $count = 0;
        while ($iterator->valid()) {
            $count++;
            $iterator->next();
        }
        return $count;
    }

    /**
     * @param callable(T): bool $callback
     */
    public function any(?callable $callback = null): bool
    {
        while ($this->valid()) {
            if ($callback === null) {
                return true;
            }

            $current = $this->current();
            assert($current !== null);

            if ($callback($current)) {
                return true;
            }
            $this->next();
        }
        return false;
    }

    /**
     * @param callable(T): bool $callback
     */
    public function all(callable $callback): bool
    {
        while ($this->valid()) {
            $current = $this->current();
            assert($current !== null);

            if (!$callback($current)) {
                return false;
            }
            $this->next();
        }
        return true;
    }

    /**
     * @param  callable(T): bool $callback
     * @return ?T
     */
    public function find(callable $callback): mixed
    {
        while ($this->valid()) {
            $current = $this->current();
            assert($current !== null);

            if ($callback($current)) {
                return $this->current();
            }
            $this->next();
        }
        return null;
    }

    /**
     * @param  callable(T): bool $callback
     * @return ?T
     */
    public function findLast(callable $callback): mixed
    {
        $last = null;
        while ($this->valid()) {
            $current = $this->current();
            assert($current !== null);

            if ($callback($current)) {
                $last = $current;
            }
            $this->next();
        }
        return $last;
    }

    /**
     * @return ?T
     */
    public function last(): mixed
    {
        $last = null;
        while ($this->valid()) {
            $last = $this->current();
            $this->next();
        }
        return $last;
    }

    /**
     * Returns the first element of the iterator, or null if the iterator is empty.
     * This does not advance or rewind the iterator.
     *
     * @return ?T
     */
    public function first(): mixed
    {
        return $this->valid() ? $this->current() : null;
    }

    public function position(): int
    {
        return $this->position;
    }

    /**
     * @param  Iter<T> ...$iterators
     * @return Chain<T>
     */
    public function chain(Iter ...$iterators): Chain
    {
        return new Chain($this, ...$iterators);
    }

    /**
     * @return Enumerator<T>
     */
    public function enumerate(): Enumerator
    {
        return new Enumerator($this);
    }

    /**
     * @param  Iter<T> ...$iterators
     * @return Zip<T>
     */
    public function zip(Iter ...$iterators): Zip
    {
        return new Zip($this, ...$iterators);
    }

    /**
     * @param  Iter<T> ...$iterators
     * @return Interleave<T>
     */
    public function interleave(Iter ...$iterators): Interleave
    {
        return new Interleave($this, ...$iterators);
    }

    /**
     * @param callable(T): void $callback
     */
    public function forEach(callable $callback): void
    {
        while ($this->valid()) {
            $current = $this->current();
            assert($current !== null);

            $callback($current);
            $this->next();
        }
    }

    /**
     * @template U
     * @param    callable(T): (Iterator<U>|array<U>) $callback
     * @return   Flatten<U>
     */
    public function flatMap(callable $callback): Flatten
    {
        /**
 * @var Flatten<U> $result
*/
        $result = $this->map($callback)->flatten();
        return $result;
    }

    /**
     * @return Flatten<mixed>
     */
    public function flatten(): Flatten
    {
        return new Flatten($this);
    }

    /**
     * @return Cycle<T>
     */
    public function cycle(): Cycle
    {
        return new Cycle($this);
    }

    /**
     * @param  callable(T): void $callback
     * @return Iter<T>
     */
    public function inspect(callable $callback): Iter
    {
        return $this->map(
            function ($value) use (&$callback) {
                $callback($value);
                return $value;
            }
        );
    }

    /**
     * @return ?T
     */
    public function current(): mixed
    {
        return $this->iterator->current();
    }

    public function next(): void
    {
        $this->position++;
        $this->iterator->next();
    }

    public function key(): int
    {
        return $this->position;
    }

    public function valid(): bool
    {
        return $this->iterator->valid();
    }

    public function rewind(): void
    {
        $this->iterator->rewind();
    }
}
