<?php

namespace Xenira\IterTools\Iter;

use Xenira\IterTools\IterToolsIterator;

class Take extends IterToolsIterator
{
    public function __construct(private IterToolsIterator $iterator, private int $n)
    {
        parent::__construct($iterator);
    }

    public function valid(): bool
    {
        return parent::valid() && $this->n > 0;
    }

    public function next(): void
    {
        parent::next();
        $this->n--;
    }

    public function skip(int $n): IterToolsIterator
    {
        parent::validateSkip($n);

        $this->iterator->skip($n);
        $this->n -= $n;
        return $this;
    }
}
