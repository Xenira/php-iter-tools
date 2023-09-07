<?php

namespace Xenira\IterTools\Iter;

use Closure;
use Xenira\IterTools\IterToolsIterator;

class Map extends IterToolsIterator
{
    private Closure $callback;

    public function __construct(private IterToolsIterator $iterator, callable $callback)
    {
        $this->callback = Closure::fromCallable($callback);
        parent::__construct($iterator);
    }

    public function current(): mixed
    {
        return ($this->callback)(parent::current());
    }

    public function skip(int $n): IterToolsIterator
    {
        parent::validateSkip($n);

        $this->iterator->skip($n);
        return $this;
    }
}
