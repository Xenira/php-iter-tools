<?php

namespace Xenira\IterTools\Iter;

use Closure;
use Xenira\IterTools\Iter;

/**
 * Class Map
 *
 * @package Xenira\IterTools\Iter
 * @template T
 * @template U
 * @template-extends Iter<T>
 */
class Map extends Iter
{
    private Closure $callback;

    /**
     * Map constructor.
     *
     * @param Iter<T> $iterator
     * @param callable(T): U $callback
     */
    public function __construct(Iter $iterator, callable $callback)
    {
        $this->callback = Closure::fromCallable($callback);
        parent::__construct($iterator);
    }

    /**
     * @return ?U
     */
    public function current(): mixed
    {
        return ($this->callback)(parent::current());
    }
}
