<?php

namespace Xenira\IterTools;

use Iterator;
use Xenira\IterTools\Iter\Chain;
use Xenira\IterTools\Iter\Enumerator;
use Xenira\IterTools\Iter\Filter;
use Xenira\IterTools\Iter\Map;
use Xenira\IterTools\Iter\Take;
use Xenira\IterTools\Iter\TakeWhile;
use Xenira\IterTools\Iter\Zip;

/**
 * @implements Iterator<mixed,mixed>
 */
abstract class IterToolsIterator implements Iterator
{
    protected int $position = 0;

    protected function  __construct(private ?IterToolsIterator $iterator) {
        assert($iterator !== null, "If you see this, you're doing something wrong. This constructor should never be called directly. Try overriding it instead.");
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

    abstract public function skip(int $n): IterToolsIterator;

    protected function validateSkip(int $n): void {
        if ($n < 0) {
            throw new \InvalidArgumentException("n must be greater than or equal to 0");
        }
    }
    /**
     * @param callable(): mixed $callback
     */
    public function skipWhile(callable $callback): IterToolsIterator {
        while ($this->valid() && $callback($this->current())) {
            $this->skip(1);
        }
        return $this;
    }

    /**
     * @param int $start inclusive
     * @param int $end exclusive
     * @return IterToolsIterator<TValue>
     */
    public function slice(int $start, int $end): IterToolsIterator {
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
        $count = 0;
        while ($this->valid()) {
            if ($callback === null || $callback($this->current())) {
                $count++;
            }
            $this->next();
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

    public function chain(IterToolsIterator ...$iterators): IterToolsIterator {
        $result = $this;
        foreach ($iterators as $iterator) {
            $result = new Chain($result, $iterator);
        }
        return $result;
    }

    public function enumerate(): Enumerator {
        return new Enumerator($this);
    }

    public function zip(IterToolsIterator ...$iterators): Zip {
        return new Zip($this, ...$iterators);
    }

    public function interleave(IterToolsIterator ...$iterators): Iterator {
        $iterators = array_merge([$this], $iterators);
        throw new \Exception("Not implemented");
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
        throw new \Exception("Not implemented");
    }

    public function flatten(): Iterator {
        throw new \Exception("Not implemented");
    }
    /**
     * @param callable(): mixed $callback
     */
    public function inspect(callable $callback): Iterator {
        $callback($this->current());
    }

    public function current()
    {
        assert($this->iterator !== null, "this->iterator must not be null");
        return $this->iterator->current();
    }

    public function next(): void
    {
        assert($this->iterator !== null, "this->iterator must not be null");
        $this->position++;
        $this->iterator->next();
    }

    public function key()
    {
        assert($this->iterator !== null, "this->iterator must not be null");
        $this->iterator->key();
    }

    public function valid(): bool
    {
        assert($this->iterator !== null, "this->iterator must not be null");
        return $this->iterator->valid();
    }

    public function rewind(): void
    {
        assert($this->iterator !== null, "this->iterator must not be null");
        $this->iterator->rewind();
    }
}
