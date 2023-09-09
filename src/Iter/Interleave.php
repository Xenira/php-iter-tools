<?php

namespace Xenira\IterTools\Iter;

use Xenira\IterTools\ArrayIterator;
use Xenira\IterTools\Iter;

class Interleave extends Iter
{
    private bool $end = false;

    public function __construct(Iter $iterator, Iter ...$interleaved)
    {
        $interleaved = (new ArrayIterator([$iterator, ...$interleaved]))->filter(fn($i) => $i->valid());
        parent::__construct($interleaved->cycle());
    }

    public function next(): void
    {
        if ($this->end) {
            return;
        }

        parent::current()->next();
        parent::next();

        $this->end = !parent::valid();
    }

    public function current()
    {
        return parent::current()->current();
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
