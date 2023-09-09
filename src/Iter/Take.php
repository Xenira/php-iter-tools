<?php

namespace Xenira\IterTools\Iter;

use Xenira\IterTools\Iter;

/**
 * Class Take
 *
 * @package Xenira\IterTools\Iter
 * @template T
 * @template-extends Iter<T>
 */
class Take extends Iter
{
    /**
     * Take constructor.
     *
     * @param Iter<T> $iterator
     * @param int $n
     */
    public function __construct(Iter $iterator, private int $n)
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

    public function skip(int $n): Iter
    {
        parent::skip($n);
        $this->n -= $n;
        return $this;
    }
}
