<?php

namespace Xenira\IterTools\Iter;

use Closure;
use Xenira\IterTools\Iter;

/**
 * Class TakeWhile
 *
 * @package Xenira\IterTools\Iter
 * @template T
 * @template-extends Iter<T>
 */
class TakeWhile extends Iter
{
    private Closure $callback;
    private bool $done = false;
    private bool $tested = false;

    /**
     * TakeWhile constructor.
     *
     * @param Iter<T> $iterator
     * @param callable(T): bool $callback
     */
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
