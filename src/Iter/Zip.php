<?php

namespace Xenira\IterTools\Iter;

use Xenira\IterTools\IterToolsIterator;

class Zip extends IterToolsIterator
{
    /** @var list<IterToolsIterator> */
    private array $zipped;

    public function __construct(private IterToolsIterator $iterator, IterToolsIterator ...$zipped)
    {
        parent::__construct($iterator);
        $this->zipped = $zipped;
    }

    public function current(): array
    {
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

    public function skip(int $n): IterToolsIterator
    {
        parent::validateSkip($n);

        $this->iterator->skip($n);
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
