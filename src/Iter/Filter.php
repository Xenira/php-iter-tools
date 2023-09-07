<?php

namespace Xenira\IterTools\Iter;

use Closure;
use Xenira\IterTools\IterToolsIterator;

class Filter extends IterToolsIterator {
    private Closure $callback;
    private bool $end = false;

    public function __construct(private IterToolsIterator $iterator, callable $callback) {
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

    public function skip(int $n): IterToolsIterator
    {
        parent::validateSkip($n);

        for ($i = 0; $i < $n; $i++) {
            if (!$this->valid()) {
                break;
            }
            $this->next();
        }
        return $this;
    }
}
