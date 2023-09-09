<?php

namespace Xenira\IterTools\Iter;

use Xenira\IterTools\Iter;

/**
 * Class Zip
 *
 * @package Xenira\IterTools\Iter
 * @template T
 * @template-extends Iter<T>
 */
class Zip extends Iter
{
    /** @var list<Iter<T>> */
    private array $zipped;

    /**
     * Zip constructor.
     *
     * @param Iter<T> $iterator
     * @param Iter<T> ...$zipped
     */
    public function __construct(Iter $iterator, Iter ...$zipped)
    {
        parent::__construct($iterator);
        $this->zipped = array_values($zipped);
    }

    /**
     * @return array<int, ?T>
     */
    public function current(): ?array
    {
        if (!parent::valid()) {
            return null;
        }

        $current = [parent::current()];
        foreach ($this->zipped as $zipped) {
            $current[] = $zipped->current();
        }
        return $current;
    }

    public function next(): void
    {
        parent::next();
        foreach ($this->zipped as $zipped) {
            $zipped->next();
        }
    }

    public function rewind(): void
    {
        parent::rewind();
        foreach ($this->zipped as $zipped) {
            $zipped->rewind();
        }
    }

    /**
     * @param int $n
     * @return Iter<T>
     */
    public function skip(int $n): Iter
    {
        parent::skip($n);
        foreach ($this->zipped as $zipped) {
            $zipped->skip($n);
        }
        return $this;
    }

    public function valid(): bool
    {
        return parent::valid() || array_reduce($this->zipped, fn ($carry, $zipped) => $carry && $zipped->valid(), true);
    }
}
