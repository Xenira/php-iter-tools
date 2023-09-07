<?php

namespace Xenira\IterTools\Iter;

use Xenira\IterTools\IterToolsIterator;

class Chain extends IterToolsIterator
{
    private ?IterToolsIterator $initialIterator = null;

    public function __construct(private IterToolsIterator $iterator, private ?IterToolsIterator $chained)
    {
        parent::__construct($iterator);
        $this->initialIterator = $iterator;
    }

    public function next(): void
    {
        if (!parent::valid() && $this->chained !== null) {
            $this->iterator = $this->chained;
            $this->chained = null;
        }

        parent::next();

        if ($this->chained !== null && !parent::valid()) {
            $this->iterator = $this->chained;
            $this->chained = null;
        }
    }

    public function rewind(): void
    {
        if ($this->chained === null) {
            $this->chained = $this->iterator;
            $this->iterator = $this->initialIterator;
        }

        $this->chained->rewind();
        parent::rewind();
    }

    public function skip(int $n): IterToolsIterator
    {
        parent::validateSkip($n);

        $currentPosition = parent::position();
        $this->currentIterator->skip($n);
        if ($this->chained !== null && !parent::valid()) {
            $diff = parent::position() - $currentPosition - $n;
            if ($diff > 0) {
                $this->chained->skip($diff);
            }
            $this->iterator = $this->chained;
        }

        return $this;
    }
}
