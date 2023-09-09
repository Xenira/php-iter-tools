<?php

namespace Xenira\IterTools\Iter;

use Xenira\IterTools\ArrayIterator;
use Xenira\IterTools\Iter;

/**
 * Class Interleave
 *
 * @package Xenira\IterTools\Iter
 *
 * @template         T
 * @template-extends Iter<Iter<T>>
 */
class Interleave extends Iter
{
    private bool $end = false;

    /**
     * Interleave constructor.
     *
     * @param Iter<T> $iterator
     * @param Iter<T> ...$interleaved
     */
    public function __construct(Iter $iterator, Iter ...$interleaved)
    {
        $interleaved = (new ArrayIterator(array_values([$iterator, ...$interleaved])))
            ->filter(fn($i) => $i->valid());
        parent::__construct($interleaved->cycle());
    }

    public function next(): void
    {
        if ($this->end) {
            return;
        }

        $current = parent::current();
        if ($current !== null) {
            $current->next();
        }
        parent::next();

        $this->end = !parent::valid();
    }

    /**
     * @return T|null
     */
    public function current(): mixed
    {
        $current = parent::current();
        if ($current === null) {
            $this->end = true;
            return null;
        }

        return $current->current();
    }

    public function valid(): bool
    {
        return !$this->end;
    }

    public function rewind(): void
    {
        $this->end = false;
        parent::forEach(fn($i) => $i->rewind());
        parent::rewind();
    }
}
