<?php

namespace Xenira\IterTools;

use Xenira\IterTools\Iter;

class ArrayIterator extends Iter
{
    private $array;

    public function __construct(array $array)
    {
        $this->array = $array;
    }

    public function current()
    {
        return $this->array[$this->position];
    }

    public function next(): void
    {
        $this->position++;
    }

    public function key()
    {
        return $this->position;
    }

    public function valid(): bool
    {
        return isset($this->array[$this->position]);
    }

    public function rewind(): void
    {
        $this->position = 0;
    }

    public function skip(int $n): Iter
    {
        parent::validateSkip($n);
        $this->position += $n;
        return $this;
    }
}
