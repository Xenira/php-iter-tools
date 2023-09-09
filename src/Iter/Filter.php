<?php

namespace Xenira\IterTools\Iter;

use Closure;
use Xenira\IterTools\Iter;

/**
 * Class Filter
 *
 * @package Xenira\IterTools\Iter
 * @template T
 * @template-extends Iter<T>
 */
class Filter extends Iter {
    private Closure $callback;
    private bool $end = false;

    /**
     * Filter constructor.
     *
     * @param Iter<T> $iterator
     * @param callable(T): bool $callback
     */
    public function __construct(Iter $iterator, callable $callback) {
        $this->callback = Closure::fromCallable($callback);
        parent::__construct($iterator);

        $this->advance();
    }

    public function next(): void {
        parent::next();
        $this->advance();
    }

    public function valid(): bool {
        return !$this->end && parent::valid();
    }

    public function rewind(): void {
        $this->end = false;
        parent::rewind();
        $this->advance();
    }

    private function advance(): void {
        $valid = parent::valid();
        while ($valid && !($this->callback)(parent::current())) {
            parent::next();
            $valid = parent::valid();
        }

        if (!$valid) {
            $this->end = true;
        }
    }
}
