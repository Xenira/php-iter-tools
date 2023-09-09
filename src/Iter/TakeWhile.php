<?php

namespace Xenira\IterTools\Iter;

use Closure;
use Xenira\IterTools\Iter;

class TakeWhile extends Iter
{
    private Closure $callback;
    private bool $done = false;
    private bool $tested = false;

    public function __construct(Iter $iterator, callable $callback)
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
}
