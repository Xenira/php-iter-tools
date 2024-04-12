<?php

namespace Xenira\IterTools;

use InvalidArgumentException;
use PHPUnit\Framework\TestCase;

class RSTest extends TestCase
{
    private \ArrayIter $iterator;

    public function setUp(): void
    {
        $array = [1, 2, 3, 4, 5];
        $this->iterator = new \ArrayIter($array);
    }

    public function testClassConstructor(): void
    {
        $this->assertInstanceOf(\ArrayIter::class, $this->iterator);
    }

    public function testCollect(): void
    {
        $this->assertEquals([1, 2, 3, 4, 5], $this->iterator->collect());
    }

    public function testCount(): void
    {
        $this->assertEquals(5, $this->iterator->count());
    }

    public function testLast(): void
    {
        $this->assertEquals(5, $this->iterator->last());
    }

    public function testNth(): void
    {
        $this->assertEquals(3, $this->iterator->nth(2));
    }

    public function testChain(): void
    {
        $iterator = $this->iterator->chain(new \ArrayIter([6, 7, 8, 9, 10]));

        $this->assertEquals([1, 2, 3, 4, 5, 6, 7, 8, 9, 10], $iterator->collect());
    }

    public function testZip(): void
    {
        $iterator = $this->iterator->zip(new \ArrayIter([6, 7, 8, 9, 10]));
        $this->assertEquals([[1, 6], [2, 7], [3, 8], [4, 9], [5, 10]], $iterator->collect());
    }

    public function testZipFirstLonger(): void
    {
        // TODO: Validate that this is the correct behavior
        $iterator = $this->iterator->zip(new \ArrayIter([6, 7, 8, 9]));
        $this->assertEquals([[1, 6], [2, 7], [3, 8], [4, 9]], $iterator->collect());
    }

    public function testZipSecondLonger(): void
    {
        // TODO: Validate that this is the correct behavior
        $iterator = $this->iterator->zip(new \ArrayIter([6, 7, 8, 9, 10, 11]));
        $this->assertEquals([[1, 6], [2, 7], [3, 8], [4, 9], [5, 10]], $iterator->collect());
    }

    public function testForEach(): void
    {
        $result = 0;

        $this->iterator->forEach(function($x) use (&$result) {
            $result += $x;
        });

        $this->assertEquals(15, $result);
    }

    public function testFilter(): void
    {
        $this->assertEquals([2, 4], $this->iterator->filter(fn($x) => $x % 2 === 0)->collect());
    }

    public function testFilterMap(): void
    {
        $this->assertEquals([4, 8], $this->iterator->filterMap(fn($x) => $x % 2 === 0 ? $x * 2 : null)->collect());
    }

    public function testEnumerate(): void
    {
        $this->assertEquals([[0, 1], [1, 2], [2, 3], [3, 4], [4, 5]], $this->iterator->enumerate()->collect());
    }

    public function testSkipWhile(): void
    {
        $this->assertEquals([3, 4, 5], $this->iterator->skipWhile(fn($x) => $x < 3)->collect());
    }

    public function testTakeWhile(): void
    {
        $this->assertEquals([1, 2], $this->iterator->takeWhile(fn($x) => $x < 3)->collect());
    }

    public function testMapWhile(): void
    {
        $this->assertEquals([2, 4], $this->iterator->mapWhile(fn($x) => $x < 3 ? $x * 2 : null)->collect());
    }

    public function testSkip(): void
    {
        $this->assertEquals([4, 5], $this->iterator->skip(3)->collect());
    }

    public function testTake(): void
    {
        $this->assertEquals([1, 2], $this->iterator->take(2)->collect());
    }

    public function testFlatMap(): void
    {
        $this->assertEquals([1, 1, 2, 2, 3, 3, 4, 4, 5, 5], $this->iterator->flatMap(fn($x) => [$x, $x])->collect());
    }

    public function testFlatten(): void
    {
        $this->assertEquals([1, 2, 3, 4, 5], $this->iterator->flatten()->collect());
        $array = [1, [2, 3], [4, [5]]];
        $iterator = new \ArrayIter($array);
        $this->assertEquals([1, 2, 3, 4, [5]], $iterator->flatten()->collect());
    }

    public function testFuse(): void
    {
        $array = [1, 2, 3, null, 4, 5];
        $iterator = new \ArrayIter($array);
        $this->assertEquals([1, 2, 3], $iterator->fuse()->collect());
    }

    public function testInspect(): void
    {
        $result = 0;

        $this->iterator->inspect(function ($x) use (&$result) {
          $result += $x;
        });

        $this->assertEquals(0, $result);
        $this->assertEquals([1, 2, 3, 4, 5], $this->iterator->collect());
        $this->assertEquals(15, $result);
    }

    public function testCollectInto(): void
    {
        // TODO: Does not accept an empty array
        $result = [0];
        $this->iterator->collectInto($result);
        $this->assertEquals([0, 1, 2, 3, 4, 5], $result);
        $this->iterator->collectInto($result);
        $this->assertEquals([0, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5], $result);
    }

    public function testPartition(): void
    {
        $result = $this->iterator->partition(fn($x) => $x % 2 === 0);
        $this->assertEquals([[2, 4], [1, 3, 5]], $result);
    }

    public function testFold(): void
    {
        $result = $this->iterator->fold(0, fn($acc, $x) => $acc + $x);
        $this->assertEquals(15, $result);

        $result = $this->iterator->fold(1, fn($acc, $x) => $acc * $x);
        $this->assertEquals(120, $result);
    }

    public function testReduce(): void
    {
        $result = $this->iterator->reduce(fn($acc, $x) => $acc + $x);
        $this->assertEquals(15, $result);

        $result = $this->iterator->reduce(fn($acc, $x) => $acc * $x);
        $this->assertEquals(120, $result);
    }

    public function testAll(): void
    {
        $this->assertTrue($this->iterator->all(fn($x) => $x > 0));
        $this->assertFalse($this->iterator->all(fn($x) => $x > 1));
        $this->assertTrue($this->iterator->all(fn($x) => $x < 6));
        $this->assertFalse($this->iterator->all(fn($x) => $x < 5));
    }

    public function testAny(): void
    {
        $this->assertTrue($this->iterator->any(fn($x) => $x > 4));
        $this->assertFalse($this->iterator->any(fn($x) => $x > 5));
        $this->assertTrue($this->iterator->any(fn($x) => $x < 5));
        $this->assertFalse($this->iterator->any(fn($x) => $x < 1));
    }

    public function testFind(): void
    {
        $this->assertEquals(3, $this->iterator->find(fn($x) => $x === 3));
        $this->assertNull($this->iterator->find(fn($x) => $x === 6));
    }

    public function testFindMap(): void
    {
        $this->assertEquals(6, $this->iterator->findMap(fn($x) => $x === 3 ? $x * 2 : null));
        $this->assertNull($this->iterator->findMap(fn($x) => $x === 6 ? $x * 2 : null));
    }

    public function testPosition(): void
    {
        $this->assertEquals(2, $this->iterator->position(fn($x) => $x === 3));
        $this->assertNull($this->iterator->position(fn($x) => $x === 6));
    }

    public function testMax(): void
    {
        $this->assertEquals(5, $this->iterator->max());
        $this->assertNull((new \ArrayIter([]))->max());
    }

    public function testMin(): void
    {
        $this->assertEquals(1, $this->iterator->min());
        $this->assertNull((new \ArrayIter([]))->min());
    }

    public function testMaxByKey(): void
    {
        $this->assertEquals(5, $this->iterator->maxByKey(fn($x) => $x * 2));
        $this->assertEquals(1, $this->iterator->maxByKey(fn($x) => $x * -2));
        $this->assertNull((new \ArrayIter([]))->maxByKey(fn($x) => $x * 2));
    }

    public function testMaxBy(): void
    {
        $this->assertEquals(5, $this->iterator->maxBy(fn($x, $y) => $x <=> $y));
        $this->assertEquals(1, $this->iterator->maxBy(fn($x, $y) => $y <=> $x));
        $this->assertNull((new \ArrayIter([]))->maxBy(fn($x, $y) => $x <=> $y));
    }

    public function testMinByKey(): void
    {
        $this->assertEquals(1, $this->iterator->minByKey(fn($x) => $x * 2));
        $this->assertEquals(5, $this->iterator->minByKey(fn($x) => $x * -2));
        $this->assertNull((new \ArrayIter([]))->minByKey(fn($x) => $x * 2));
    }

    public function testMinBy(): void
    {
        $this->assertEquals(1, $this->iterator->minBy(fn($x, $y) => $x <=> $y));
        $this->assertEquals(5, $this->iterator->minBy(fn($x, $y) => $y <=> $x));
        $this->assertNull((new \ArrayIter([]))->minBy(fn($x, $y) => $x <=> $y));
    }

    public function testCmp(): void
    {
        $this->assertEquals(-1, $this->iterator->cmp(new \ArrayIter([1, 2, 3, 4, 6])));
        $this->assertEquals(0, $this->iterator->cmp(new \ArrayIter([1, 2, 3, 4, 5])));
        $this->assertEquals(1, $this->iterator->cmp(new \ArrayIter([1, 2, 3, 4, 4])));
    }

    public function testPartialCmp(): void
    {
        $this->assertEquals(-1, $this->iterator->partialCmp(new \ArrayIter([1, 2, 3, 4, 6])));
        $this->assertEquals(0, $this->iterator->partialCmp(new \ArrayIter([1, 2, 3, 4, 5])));
        $this->assertEquals(1, $this->iterator->partialCmp(new \ArrayIter([1, 2, 3, 4, 4])));
        $this->assertNull($this->iterator->partialCmp(new \ArrayIter([1, 2, 3, 4, null])));
    }

    public function testEq(): void
    {
        $this->assertTrue($this->iterator->eq(new \ArrayIter([1, 2, 3, 4, 5])));
        $this->assertFalse($this->iterator->eq(new \ArrayIter([1, 2, 3, 4, 6])));
    }

    public function testNe(): void
    {
        $this->assertTrue($this->iterator->ne(new \ArrayIter([1, 2, 3, 4, 6])));
        $this->assertFalse($this->iterator->ne(new \ArrayIter([1, 2, 3, 4, 5])));
    }

    public function testLt(): void
    {
        $this->assertTrue($this->iterator->lt(new \ArrayIter([1, 2, 3, 4, 6])));
        $this->assertFalse($this->iterator->lt(new \ArrayIter([1, 2, 3, 4, 5])));
        $this->assertFalse($this->iterator->lt(new \ArrayIter([1, 2, 3, 4, 4])));
    }

    public function testLe(): void
    {
        $this->assertTrue($this->iterator->le(new \ArrayIter([1, 2, 3, 4, 6])));
        $this->assertTrue($this->iterator->le(new \ArrayIter([1, 2, 3, 4, 5])));
        $this->assertFalse($this->iterator->le(new \ArrayIter([1, 2, 3, 4, 4])));
    }

    public function testGt(): void
    {
        $this->assertFalse($this->iterator->gt(new \ArrayIter([1, 2, 3, 4, 6])));
        $this->assertFalse($this->iterator->gt(new \ArrayIter([1, 2, 3, 4, 5])));
        $this->assertTrue($this->iterator->gt(new \ArrayIter([1, 2, 3, 4, 4])));
    }

    public function testGe(): void
    {
        $this->assertFalse($this->iterator->ge(new \ArrayIter([1, 2, 3, 4, 6])));
        $this->assertTrue($this->iterator->ge(new \ArrayIter([1, 2, 3, 4, 5])));
        $this->assertTrue($this->iterator->ge(new \ArrayIter([1, 2, 3, 4, 4])));
    }

    public function testFirst()
    {
        $this->assertEquals(1, $this->iterator->first());
    }
}
