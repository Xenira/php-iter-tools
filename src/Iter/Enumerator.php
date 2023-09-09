<?php

namespace Xenira\IterTools\Iter;

use Xenira\IterTools\Iter;

/**
 * Class Enumerator
 *
 * @package Xenira\IterTools\Iter
 * @template T
 * @template-extends Iter<T>
 */
class Enumerator extends Iter
{
    private int $index = 0;

    /**
     * Enumerator constructor.
     *
     * @param Iter<T> $iterator
     */
    public function __construct(Iter $iterator)
    {
        parent::__construct($iterator);
    }

    /**
     * @return array{0: int, 1: T}
     */
    public function current(): ?array
    {
        $current = parent::current();
        if ($current === null) {
            return null;
        }

        return [$this->index, $current];
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

    /**
     * @param int $n
     * @return Iter<T>
     */
    public function skip(int $n): Iter
    {
        parent::skip($n);
        $this->index += $n;
        return $this;
    }
}
