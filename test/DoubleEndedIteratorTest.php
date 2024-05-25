<?php

declare(strict_types=1);

use PHPUnit\Framework\TestCase;

final class DoubleEndedIteratorTest extends TestCase
{
    public function testNextBack(): void
    {
        $iter = \ArrayIter::new([1, 2, 3, 4, 5, 6]);
        $this->assertEquals(1, $iter->next());
        $this->assertEquals(6, $iter->nextBack());
        $this->assertEquals(5, $iter->nextBack());
        $this->assertEquals(2, $iter->next());
        $this->assertEquals(3, $iter->next());
        $this->assertEquals(4, $iter->next());
        $this->assertEquals(null, $iter->next());
        $this->assertEquals(null, $iter->nextBack());
    }

    public function testNthBack(): void
    {
        $iter = \ArrayIter::new([1, 2, 3]);
        $this->assertEquals(1, $iter->nthBack(2));

        $iter = \ArrayIter::new([1, 2, 3]);
        $this->assertEquals(2, $iter->nthBack(1));
        $this->assertEquals(null, $iter->nthBack(1));

        $iter = \ArrayIter::new([1, 2, 3]);
        $this->assertEquals(null, $iter->nthBack(10));
    }

    public function testRfold()
    {
        $iter = \ArrayIter::new([1, 2, 3]);
        $sum = $iter->rfold(0, fn($acc, $x) => $acc + $x);
        $this->assertEquals(6, $sum);

        $iter = \ArrayIter::new([1, 2, 3, 4, 5]);
        $zero = '0';
        $result = $iter->rfold($zero, fn($acc, $x) => sprintf("(%s + %s)", $x, $acc));
        $this->assertEquals("(1 + (2 + (3 + (4 + (5 + 0)))))", $result);
    }

    public function testRfind()
    {
        $iter = \ArrayIter::new([1, 2, 3]);
        $this->assertEquals(2, $iter->rfind(fn($x) => $x === 2));
        $this->assertEquals(1, $iter->nextBack());
    }
}
