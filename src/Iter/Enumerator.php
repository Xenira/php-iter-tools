<?php

namespace Xenira\IterTools\Iter;

use Xenira\IterTools\IterToolsIterator;

class Enumerator extends IterToolsIterator
{
    private int $index = 0;

    public function __construct(private IterToolsIterator $iterator)
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

    public function skip(int $n): IterToolsIterator
    {
        parent::validateSkip($n);

        $this->iterator->skip($n);
        $this->index += $n;
        return $this;
    }
}
