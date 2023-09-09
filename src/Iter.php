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
 * @implements Iterator<mixed,mixed>
 */
abstract class Iter implements Iterator
{
    protected int $position = 0;

    protected function  __construct(private Iterator $iterator) {
    }

    /**
     * @param callable(): mixed $callback
     */
    public function filter(callable $callback): Filter {
        return new Filter($this, $callback);
    }
    /**
     * @param callable(): mixed $callback
     */
    public function map(callable $callback): Map {
        return new Map($this, $callback);
    }
    /**
     * @param callable(): mixed $callback
     * @param mixed $initial
     */
    public function reduce(callable $callback, $initial = null): mixed {
        while ($this->valid()) {
            $initial = $callback($initial, $this->current());
            $this->next();
        }
        return $initial;
    }

    public function take(int $n): Take {
        return new Take($this, $n);
    }
    /**
     * @param callable(): mixed $callback
     */
    public function takeWhile(callable $callback): TakeWhile {
        return new TakeWhile($this, $callback);
    }

    public function skip(int $n): Iter {
        $this->validateSkip($n);

        while ($n > 0) {
            $this->next();
            $n--;
        }

        return $this;
    }

    protected function validateSkip(int $n): void {
        if ($n < 0) {
            throw new \InvalidArgumentException("n must be greater than or equal to 0");
        }
    }
    /**
     * @param callable(): mixed $callback
     */
    public function skipWhile(callable $callback): Iter {
        while ($this->valid() && $callback($this->current())) {
            $this->skip(1);
        }
        return $this;
    }

    /**
     * @param int $start inclusive
     * @param int $end exclusive
     * @return Iter<TValue>
     */
    public function slice(int $start, int $end): Iter {
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
     * @return TValue[]
     */
    public function collect(): array {
        $result = [];
        while ($this->valid()) {
            $result[] = $this->current();
            $this->next();
        }
        return $result;
    }

    public function count(?callable $callback = null): int {
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

    public function any(?callable $callback = null): bool {
        while ($this->valid()) {
            if ($callback === null) {
                return true;
            }

            if ($callback($this->current())) {
                return true;
            }
            $this->next();
        }
        return false;
    }
    /**
     * @param callable(): bool $callback
     */
    public function all(callable $callback): bool {
        while ($this->valid()) {
            if (!$callback($this->current())) {
                return false;
            }
            $this->next();
        }
        return true;
    }

    /**
     * @param callable(): bool $callback
     */
    public function find(callable $callback): mixed {
        while ($this->valid()) {
            if ($callback($this->current())) {
                return $this->current();
            }
            $this->next();
        }
        return null;
    }

    /**
     * @param callable(): bool $callback
     */
    public function findLast(callable $callback): mixed {
        $last = null;
        while ($this->valid()) {
            if ($callback($this->current())) {
                $last = $this->current();
            }
            $this->next();
        }
        return $last;
    }

    public function last(): mixed {
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
     * @return mixed
     */
    public function first(): mixed {
        return $this->valid() ? $this->current() : null;
    }

    public function position(): int {
        return $this->position;
    }

    public function chain(Iter ...$iterators): Chain {
        return new Chain($this, ...$iterators);
    }

    public function enumerate(): Enumerator {
        return new Enumerator($this);
    }

    public function zip(Iter ...$iterators): Zip {
        return new Zip($this, ...$iterators);
    }

    public function interleave(Iter ...$iterators): Interleave {
        return new Interleave($this, ...$iterators);
    }
    /**
     * @param callable(): mixed $callback
     */
    public function forEach(callable $callback): void {
        while ($this->valid()) {
            $callback($this->current());
            $this->next();
        }
    }
    /**
     * @param callable(): mixed $callback
     */
    public function flatMap(callable $callback): Iterator {
        return $this->map($callback)->flatten();
    }

    public function flatten(): Iterator {
        return new Flatten($this);
    }

    public function cycle(): Cycle {
        return new Cycle($this);
    }

    /**
     * @param callable(): mixed $callback
     */
    public function inspect(callable $callback): Iter {
        return $this->map(function ($value) use (&$callback) {
            $callback($value);
            return $value;
        });
    }

    public function current()
    {
        return $this->iterator->current();
    }

    public function next(): void
    {
        $this->position++;
        $this->iterator->next();
    }

    public function key()
    {
        $this->iterator->key();
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
