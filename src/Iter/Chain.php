<?php

namespace Xenira\IterTools\Iter;

use Xenira\IterTools\ArrayIterator;
use Xenira\IterTools\Iter;

class Chain extends Iter
{
    private bool $end = false;

    public function __construct(Iter $iterator, Iter ...$chained)
    {
        parent::__construct((new ArrayIterator(array_merge([$iterator], $chained)))->filter(fn($i) => $i->valid()));
    }


    public function current()
    {
        return parent::current()->current();
    }

    public function next(): void
    {
        if ($this->end) {
            return;
        }

        parent::current()->next();

        if (!parent::current()->valid()) {
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
        // TODO: Evaluate if this is a good idea. It would be nice to be able to rewind the chain iterator, but it would be a bit tricky to implement efficiently.
        throw new \BadMethodCallException('Chain iterator cannot be rewound');
    }
}
