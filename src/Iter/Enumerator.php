<?php

namespace Xenira\IterTools\Iter;

use Xenira\IterTools\Iter;

class Enumerator extends Iter
{
    private int $index = 0;

    public function __construct(Iter $iterator)
    {
        parent::__construct($iterator);
    }

    public function current(): array
    {
        return [$this->index, parent::current()];
    }

    public function next(): void
    {
        parent::next();
        $this->index++;
    }

    public function rewind(): void
    {
        $this->index = 0;
        parent::rewind();
    }

    public function skip(int $n): Iter
    {
        parent::skip($n);
        $this->index += $n;
        return $this;
    }
}
