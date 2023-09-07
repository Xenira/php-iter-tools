<?php

namespace Xenira\IterTools\Iter;

use Closure;
use Xenira\IterTools\IterToolsIterator;

class TakeWhile extends IterToolsIterator
{
    private Closure $callback;
    private bool $done = false;
    private bool $tested = false;

    public function __construct(private IterToolsIterator $iterator, callable $callback)
    {
        $this->callback = Closure::fromCallable($callback);
        parent::__construct($iterator);
    }

    public function valid(): bool
    {
        if ($this->done) {
            return false;
        }
        if (!$this->tested) {
            $this->done = !parent::valid() || !($this->callback)(parent::current());
            $this->tested = true;
        }

        return !$this->done;
    }

    public function next(): void
    {
        parent::next();
        $this->tested = false;
    }

    public function skip(int $n): IterToolsIterator
    {
        parent::validateSkip($n);

        $this->iterator->skip($n);
        return $this;
    }
}
