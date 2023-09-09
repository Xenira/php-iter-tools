<?php

namespace Xenira\IterTools\Iter;

use Xenira\IterTools\ArrayIterator;
use Xenira\IterTools\Iter;

/**
 * Class Chain
 *
 * @package Xenira\IterTools\Iter
 *
 * @template         T
 * @template-extends Iter<Iter<T>>
 */
class Chain extends Iter
{
    private bool $end = false;

    /**
     * Chain constructor.
     *
     * @param Iter<T> $iterator
     * @param Iter<T> ...$chained
     */
    public function __construct(Iter $iterator, Iter ...$chained)
    {
        parent::__construct((new ArrayIterator(array_values([$iterator, ...$chained])))->filter(fn($i) => $i->valid()));
    }

    /**
     * @return T|null
     */
    public function current(): mixed
    {
        if ($this->end) {
            return null;
        }

        $current = parent::current();
        if ($current === null) {
            $this->end = true;
            return null;
        }

        return $current->current();
    }

    public function next(): void
    {
        if ($this->end) {
            return;
        }
        $current = parent::current();
        if ($current === null) {
            $this->end = true;
            return;
        }

        $current->next();
        $current = parent::current();

        if ($current !== null && !$current->valid()) {
            parent::next();
        }

        $this->end = !parent::valid();
    }

    public function valid(): bool
    {
        return !$this->end && parent::valid();
    }

    public function rewind(): void
    {
        // TODO: Evaluate if this is a good idea. It would be nice to be able to rewind the chain iterator,
        //       but it would be a bit tricky to implement efficiently.
        throw new \BadMethodCallException('Chain iterator cannot be rewound');
    }
}
