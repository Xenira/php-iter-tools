<?php declare(strict_types=1);

namespace Xenira\IterTools\Iter;

use InvalidArgumentException;
use Iterator;
use Xenira\IterTools\ArrayIterator;
use Xenira\IterTools\Iter;

class Flatten extends Iter
{
    private Iterator $current;

    public function __construct(Iter $iterator, Iter ...$iterators)
    {
        parent::__construct($iterator, ...$iterators);
        $this->setCurrent(parent::current());
    }

    public function next(): void
    {
        $this->current->next();
        if (!$this->current->valid()) {
            parent::next();
            $this->setCurrent(parent::current());
        }
    }

    public function current()
    {
        return $this->current->current();
    }

    public function valid(): bool
    {
        return $this->current !== null && $this->current->valid();
    }

    public function rewind(): void
    {
        parent::rewind();
        $this->setCurrent(parent::current());
    }

    private function setCurrent(Iterator|array $current): void
    {
        if (is_array($current)) {
            $current = new ArrayIterator($current);
        } elseif (!$current instanceof Iterator) {
            throw new InvalidArgumentException('Expected Iterator or array');
        }

        $this->current = $current;
    }
}
