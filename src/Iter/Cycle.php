<?php

namespace Xenira\IterTools\Iter;

use Xenira\IterTools\Iter;

/**
 * Class Cycle
 *
 * @package Xenira\IterTools\Iter
 *
 * @template         T
 * @template-extends Iter<T>
 */
class Cycle extends Iter
{
    /**
     * Cycle constructor.
     *
     * @param Iter<T> $iterator
     */
    public function __construct(protected Iter $iterator)
    {
        parent::__construct($iterator);
    }

    public function next(): void
    {
        parent::next();
        if (!parent::valid()) {
            parent::rewind();
        }
    }

    /**
     * As cycle needs to rewind the iterator, we need to validate the skip.
     * This causes a greater runtime as valid() is called on every step.
     *
     * @param  int $n
     * @return Iter<T>
     */
    public function skip(int $n): Iter
    {
        parent::validateSkip($n);

        while ($n > 0) {
            if (!parent::valid()) {
                parent::rewind();
            }
            parent::next();
            $n--;
        }

        return $this;
    }
}
