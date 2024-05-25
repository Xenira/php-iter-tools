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
        $this->iterator = \ArrayIter::new($array);
    }

    public function testClassConstructor(): void
    {
        $this->assertInstanceOf(\ArrayIter::class, $this->iterator);
    }

    public function testNext(): void
    {
        $iter = \ArrayIter::new([1, 2, 3]);
        $this->assertEquals(1, $iter->next());
        $this->assertEquals(2, $iter->next());
        $this->assertEquals(3, $iter->next());
        $this->assertEquals(null, $iter->next());
        $this->assertEquals(null, $iter->next());
    }

    public function testSizeHint(): void
    {
        $iter = \ArrayIter::new([1, 2, 3]);
        $this->assertEquals([3, 3], $iter->sizeHint());
        $iter->next();
        $this->assertEquals([2, 2], $iter->sizeHint());

        $iter = \ArrayIter::new([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        $this->assertEquals([10, 10], $iter->sizeHint());
        $iter = $iter->filter(fn($x) => $x % 2 === 0);
        $this->assertEquals([0, 10], $iter->sizeHint());
        $iter = $iter->chain(\ArrayIter::new([15, 16, 17, 18, 19]));
        $this->assertEquals([5, 15], $iter->sizeHint());
    }

    public function testCollect(): void
    {
        $this->assertEquals([1, 2, 3, 4, 5], $this->iterator->collect());
        $this->assertEquals([], $this->iterator->collect());
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

    public function testStepBy(): void
    {
        $iterator = \ArrayIter::new([0, 1, 2, 3, 4, 5])->stepBy(2);
        $this->assertEquals(0, $iterator->next());
        $this->assertEquals(2, $iterator->next());
        $this->assertEquals(4, $iterator->next());
        $this->assertEquals(null, $iterator->next());
    }

    public function testChain(): void
    {
        $iterator = $this->iterator->chain(\ArrayIter::new([6, 7, 8, 9, 10]));

        $this->assertEquals([1, 2, 3, 4, 5, 6, 7, 8, 9, 10], $iterator->collect());
    }

    public function testZip(): void
    {
        $iterator = $this->iterator->zip(\ArrayIter::new([6, 7, 8, 9, 10]));
        $this->assertEquals([[1, 6], [2, 7], [3, 8], [4, 9], [5, 10]], $iterator->collect());
    }

    public function testZipFirstLonger(): void
    {
        // TODO: Validate that this is the correct behavior
        $iterator = $this->iterator->zip(\ArrayIter::new([6, 7, 8, 9]));
        $this->assertEquals([[1, 6], [2, 7], [3, 8], [4, 9]], $iterator->collect());
    }

    public function testZipSecondLonger(): void
    {
        // TODO: Validate that this is the correct behavior
        $iterator = $this->iterator->zip(\ArrayIter::new([6, 7, 8, 9, 10, 11]));
        $this->assertEquals([[1, 6], [2, 7], [3, 8], [4, 9], [5, 10]], $iterator->collect());
    }

    public function testMap(): void
    {
        $iterator = $this->iterator->map(fn($x) => $x * 2);
        $this->assertEquals([2, 4, 6, 8, 10], $iterator->collect());
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
        $iterator = \ArrayIter::new([1, 2, 3, 4, 5])->flatMap(fn($x) => [$x, $x]);
        $this->assertEquals([1, 1, 2, 2, 3, 3, 4, 4, 5, 5], $iterator->collect());
    }

    public function testFlatten(): void
    {
        $this->assertEquals([1, 2, 3, 4, 5], $this->iterator->flatten()->collect());
        $array = [1, [2, 3], [4, [5]]];
        $iterator = \ArrayIter::new($array);
        $this->assertEquals([1, 2, 3, 4, [5]], $iterator->flatten()->collect());
    }

    public function testInspect(): void
    {
        $result = 0;
        $iterator = \ArrayIter::new([1, 2, 3, 4, 5]);

        $iterator->inspect(function ($x) use (&$result) {
          $result += $x;
        });

        $this->assertEquals(0, $result);
        $this->assertEquals([1, 2, 3, 4, 5], $iterator->collect());
        $this->assertEquals(15, $result);
    }

    // TODO: wait for rust implementation to stabilize
    // public function testCollectInto(): void
    // {
    //     // TODO: Does not accept an empty array
    //     $result = [0];
    //     $this->iterator->collectInto($result);
    //     $this->assertEquals([0, 1, 2, 3, 4, 5], $result);
    //     \ArrayIter::new([1, 2, 3, 4, 5])->collectInto($result);
    //     $this->assertEquals([0, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5], $result);
    // }

    public function testPartition(): void
    {
        $result = $this->iterator->partition(fn($x) => $x % 2 === 0);
        $this->assertEquals([[2, 4], [1, 3, 5]], $result);
    }

    public function testFold(): void
    {
        $result = \ArrayIter::new([1, 2, 3, 4, 5])->fold(0, fn($acc, $x) => $acc + $x);
        $this->assertEquals(15, $result);

        $result = \ArrayIter::new([1, 2, 3, 4, 5])->fold(1, fn($acc, $x) => $acc * $x);
        $this->assertEquals(120, $result);
    }

    public function testReduce(): void
    {
        $result = \ArrayIter::new([1, 2, 3, 4, 5])->reduce(fn($acc, $x) => $acc + $x);
        $this->assertEquals(15, $result);

        $result = \ArrayIter::new([1, 2, 3, 4, 5])->reduce(fn($acc, $x) => $acc * $x);
        $this->assertEquals(120, $result);
    }

    public function testAll(): void
    {
        $this->assertTrue(\ArrayIter::new([1, 2, 3, 4, 5])->all(fn($x) => $x > 0));
        $this->assertFalse(\ArrayIter::new([1, 2, 3, 4, 5])->all(fn($x) => $x > 1));
        $this->assertTrue(\ArrayIter::new([1, 2, 3, 4, 5])->all(fn($x) => $x < 6));
        $this->assertFalse(\ArrayIter::new([1, 2, 3, 4, 5])->all(fn($x) => $x < 5));
    }

    public function testAny(): void
    {
        $this->assertTrue(\ArrayIter::new([1, 2, 3, 4, 5])->any(fn($x) => $x > 4));
        $this->assertFalse(\ArrayIter::new([1, 2, 3, 4, 5])->any(fn($x) => $x > 5));
        $this->assertTrue(\ArrayIter::new([1, 2, 3, 4, 5])->any(fn($x) => $x < 5));
        $this->assertFalse(\ArrayIter::new([1, 2, 3, 4, 5])->any(fn($x) => $x < 1));
    }

    public function testFind(): void
    {
        $this->assertEquals(3, $this->iterator->find(fn($x) => $x === 3));
        $this->assertNull(\ArrayIter::new([1, 2, 3, 4, 5])->find(fn($x) => $x === 6));
    }

    public function testFindMap(): void
    {
        $this->assertEquals(6, $this->iterator->findMap(fn($x) => $x === 3 ? $x * 2 : null));
        $this->assertNull(\ArrayIter::new([1, 2, 3, 4, 5])->findMap(fn($x) => $x === 6 ? $x * 2 : null));
    }

    public function testPosition(): void
    {
        $this->assertEquals(2, $this->iterator->position(fn($x) => $x === 3));
        $this->assertNull(\ArrayIter::new([1, 2, 3, 4, 5])->position(fn($x) => $x === 6));
    }

    public function testRposition(): void
    {
        $iter = \ArrayIter::new([1, 2, 3]);
        $this->assertEquals(2, $iter->rposition(fn($x) => $x === 3));
        $this->assertEquals(null, $iter->rposition(fn($x) => $x === 5));

        $iter = \ArrayIter::new([-1, 2, 3, 4]);
        $this->assertEquals(3, $iter->rposition(fn($x) => $x >= 2));
        $this->assertEquals(-1, $iter->next());
    }

    public function testMax(): void
    {
        $this->assertEquals(5, $this->iterator->max());
        $this->assertNull(\ArrayIter::new([])->max());
    }

    public function testMin(): void
    {
        $this->assertEquals(1, $this->iterator->min());
        $this->assertNull(\ArrayIter::new([])->min());
    }

    public function testMaxByKey(): void
    {
        $this->assertEquals(5, \ArrayIter::new([1,2,3,4,5])->maxByKey(fn($x) => $x * 2));
        $this->assertEquals(1, \ArrayIter::new([1,2,3,4,5])->maxByKey(fn($x) => $x * -2));
        $this->assertNull(\ArrayIter::new([])->maxByKey(fn($x) => $x * 2));
    }

    public function testMaxBy(): void
    {
        $this->assertEquals(5, \ArrayIter::new([1, 2, 3, 4, 5])->maxBy(fn($x, $y) => $x <=> $y));
        $this->assertEquals(1, \ArrayIter::new([1, 2, 3, 4, 5])->maxBy(fn($x, $y) => $y <=> $x));
        $this->assertNull(\ArrayIter::new([])->maxBy(fn($x, $y) => $x <=> $y));
    }

    public function testMinByKey(): void
    {
        $this->assertEquals(1, \ArrayIter::new([1, 2, 3, 4, 5])->minByKey(fn($x) => $x * 2));
        $this->assertEquals(5, \ArrayIter::new([1, 2, 3, 4, 5])->minByKey(fn($x) => $x * -2));
        $this->assertNull((\ArrayIter::new([]))->minByKey(fn($x) => $x * 2));
    }

    public function testMinBy(): void
    {
        $this->assertEquals(1, \ArrayIter::new([1, 2, 3, 4, 5])->minBy(fn($x, $y) => $x <=> $y));
        $this->assertEquals(5, \ArrayIter::new([1, 2, 3, 4, 5])->minBy(fn($x, $y) => $y <=> $x));
        $this->assertNull(\ArrayIter::new([])->minBy(fn($x, $y) => $x <=> $y));
    }

    public function testCmp(): void
    {
        $this->assertEquals(-1, \ArrayIter::new([1, 2, 3, 4, 5])->cmp(\ArrayIter::new([1, 2, 3, 4, 6])));
        $this->assertEquals(0, \ArrayIter::new([1, 2, 3, 4, 5])->cmp(\ArrayIter::new([1, 2, 3, 4, 5])));
        $this->assertEquals(1, \ArrayIter::new([1, 2, 3, 4, 5])->cmp(\ArrayIter::new([1, 2, 3, 4, 4])));
    }

    public function testPartialCmp(): void
    {
        $this->assertEquals(-1, \ArrayIter::new([1, 2, 3, 4, 5])->partialCmp(\ArrayIter::new([1, 2, 3, 4, 6])));
        $this->assertEquals(0, \ArrayIter::new([1, 2, 3, 4, 5])->partialCmp(\ArrayIter::new([1, 2, 3, 4, 5])));
        $this->assertEquals(1, \ArrayIter::new([1, 2, 3, 4, 5])->partialCmp(\ArrayIter::new([1, 2, 3, 4, 4])));
        $this->assertNull(\ArrayIter::new([1, 2, 3, 4, 5])->partialCmp(\ArrayIter::new([1, 2, 3, 4, null])));
    }

    public function testEq(): void
    {
        $this->assertTrue(\ArrayIter::new([1, 2, 3, 4, 5])->eq(\ArrayIter::new([1, 2, 3, 4, 5])));
        $this->assertFalse(\ArrayIter::new([1, 2, 3, 4, 5])->eq(\ArrayIter::new([1, 2, 3, 4, 6])));
    }

    public function testNe(): void
    {
        $this->assertTrue(\ArrayIter::new([1, 2, 3, 4, 5])->ne(\ArrayIter::new([1, 2, 3, 4, 6])));
        $this->assertFalse(\ArrayIter::new([1, 2, 3, 4, 5])->ne(\ArrayIter::new([1, 2, 3, 4, 5])));
    }

    public function testLt(): void
    {
        $this->assertTrue(\ArrayIter::new([1, 2, 3, 4, 5])->lt(\ArrayIter::new([1, 2, 3, 4, 6])));
        $this->assertFalse(\ArrayIter::new([1, 2, 3, 4, 5])->lt(\ArrayIter::new([1, 2, 3, 4, 5])));
        $this->assertFalse(\ArrayIter::new([1, 2, 3, 4, 5])->lt(\ArrayIter::new([1, 2, 3, 4, 4])));
    }

    public function testLe(): void
    {
        $this->assertTrue(\ArrayIter::new([1, 2, 3, 4, 5])->le(\ArrayIter::new([1, 2, 3, 4, 6])));
        $this->assertTrue(\ArrayIter::new([1, 2, 3, 4, 5])->le(\ArrayIter::new([1, 2, 3, 4, 5])));
        $this->assertFalse(\ArrayIter::new([1, 2, 3, 4, 5])->le(\ArrayIter::new([1, 2, 3, 4, 4])));
    }

    public function testGt(): void
    {
        $this->assertFalse(\ArrayIter::new([1, 2, 3, 4, 5])->gt(\ArrayIter::new([1, 2, 3, 4, 6])));
        $this->assertFalse(\ArrayIter::new([1, 2, 3, 4, 5])->gt(\ArrayIter::new([1, 2, 3, 4, 5])));
        $this->assertTrue(\ArrayIter::new([1, 2, 3, 4, 5])->gt(\ArrayIter::new([1, 2, 3, 4, 4])));
    }

    public function testGe(): void
    {
        $this->assertFalse(\ArrayIter::new([1, 2, 3, 4, 5])->ge(\ArrayIter::new([1, 2, 3, 4, 6])));
        $this->assertTrue(\ArrayIter::new([1, 2, 3, 4, 5])->ge(\ArrayIter::new([1, 2, 3, 4, 5])));
        $this->assertTrue(\ArrayIter::new([1, 2, 3, 4, 5])->ge(\ArrayIter::new([1, 2, 3, 4, 4])));
    }

    // public function testFirst()
    // {
    //     $this->assertEquals(1, $this->iterator->first());
    // }
}
