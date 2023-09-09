<?php

namespace Xenira\IterTools;

use Xenira\IterTools\Iter;

/**
 * Class ArrayIterator
 *
 * @package          Xenira\IterTools
 * @template         T
 * @template-extends Iter<T>
 */
class ArrayIterator extends Iter
{
    /**
     * @var T[]
     */
    private $array;

    /**
     * ArrayIterator constructor.
     *
     * @param list<T> $array
     */
    public function __construct(array $array)
    {
        $this->array = $array;
    }

    /**
     * @return T?
     */
    public function current(): mixed
    {
        return $this->array[$this->position];
    }

    public function next(): void
    {
        $this->position++;
    }

    public function key(): int
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

    /**
     * @param  int $n
     * @return Iter<T>
     */
    public function skip(int $n): Iter
    {
        parent::validateSkip($n);
        $this->position += $n;
        return $this;
    }
}
