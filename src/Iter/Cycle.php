<?php

namespace Xenira\IterTools\Iter;

use Xenira\IterTools\Iter;

class Cycle extends Iter
{
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
     * As cycle needs to rewind the iterator, we need to validate the skip. This causes a greater runtime as valid() is called on every step.
     * @param int $n
     * @return Iter
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
