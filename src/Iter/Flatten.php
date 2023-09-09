<?php declare(strict_types=1);

namespace Xenira\IterTools\Iter;

use InvalidArgumentException;
use Iterator;
use Xenira\IterTools\ArrayIterator;
use Xenira\IterTools\Iter;

/**
 * Class Flatten
 *
 * @package Xenira\IterTools\Iter
 * @template T
 * @template-extends Iter<Iterator<T>|T[]>
 */
class Flatten extends Iter
{
    private ?Iterator $current;

    /**
     * Flatten constructor.
     *
     * @param Iter<Iterator<T>|T[]> $iterator
     */
    public function __construct(Iter $iterator)
    {
        parent::__construct($iterator);
        $this->setCurrent(parent::current());
    }

    public function next(): void
    {
        $current = $this->current;
        if ($current === null) {
            return;
        }

        $current->next();
        if (!$current->valid()) {
            parent::next();
            $this->setCurrent(parent::current());
        }
    }

    /**
     * @return ?T
     */
    public function current(): mixed
    {
        if ($this->current === null) {
            return null;
        }

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

    /**
     * @param Iterator<T>|array<T>|null $current
     */
    private function setCurrent(Iterator|array|null $current): void
    {
        if (is_array($current)) {
            $current = new ArrayIterator($current);
        } elseif (!$current instanceof Iterator) {
            throw new InvalidArgumentException('Expected Iterator or array');
        }

        $this->current = $current;
    }
}
