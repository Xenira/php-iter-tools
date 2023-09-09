<?php

namespace Xenira\IterTools\Iter;

use Closure;
use Xenira\IterTools\Iter;

class Map extends Iter
{
    private Closure $callback;

    public function __construct(Iter $iterator, callable $callback)
    {
        $this->callback = Closure::fromCallable($callback);
        parent::__construct($iterator);
    }

    public function current(): mixed
    {
        return ($this->callback)(parent::current());
    }
}
