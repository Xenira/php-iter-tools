<?php

namespace Xenira\IterTools;

use InvalidArgumentException;
use PHPUnit\Framework\TestCase;

class ArrayIteratorTest extends TestCase {
    private $iterator;

    public function setUp(): void
    {
        $array = [1, 2, 3, 4, 5];
        $this->iterator = new ArrayIterator($array);
    }

    public function testClassConstructor(): void
    {
        $this->assertInstanceOf(ArrayIterator::class, $this->iterator);
        $this->assertEquals([1, 2, 3, 4, 5], iterator_to_array($this->iterator));
    }

    public function testMap(): void
    {
        $this->iterator = $this->iterator->map(fn($x) => $x * 2);

        $this->assertEquals([2, 4, 6, 8, 10], $this->iterator->collect());
    }

    public function testFilter(): void
    {
        $this->iterator = $this->iterator->filter(fn($x) => $x % 2 === 0);

        $this->assertEquals([2, 4], $this->iterator->collect());
    }

    public function testMapFilter(): void
    {
        $this->iterator = $this->iterator->map(fn($x) => $x * 2)->filter(fn($x) => $x > 4);

        $this->assertEquals([6, 8, 10], $this->iterator->collect());
    }

    public function testReduce(): void
    {
        $this->assertEquals(15, $this->iterator->reduce(fn($acc, $x) => $acc + $x));
        $this->iterator->rewind();
        $this->assertEquals(16, $this->iterator->reduce(fn($acc, $x) => $acc + $x, 1));
    }

    public function testTake(): void
    {
        $this->iterator = $this->iterator->take(3);

        $this->assertEquals([1, 2, 3], $this->iterator->collect());
    }

    public function testTakeWhile(): void
    {
        $this->iterator = $this->iterator->takeWhile(fn($x) => $x < 4);

        $this->assertEquals([1, 2, 3], $this->iterator->collect());
    }

    public function testSkip(): void
    {
        $this->iterator = $this->iterator->skip(3);

        $this->assertEquals([4, 5], $this->iterator->collect());
        $this->expectException(InvalidArgumentException::class);
        $this->iterator->skip(-1);
    }

    public function testSkipWhile(): void
    {
        $this->iterator = $this->iterator->skipWhile(fn($x) => $x < 4);

        $this->assertEquals([4, 5], $this->iterator->collect());
    }

    public function testSlice(): void
    {
        $this->iterator = $this->iterator->slice(1, 4);

        $this->assertEquals([2, 3, 4], $this->iterator->collect());

        $this->expectException(InvalidArgumentException::class);
        $this->expectExceptionMessage('start must be greater than or equal to 0');
        $this->iterator = $this->iterator->slice(-3, 4);

        $this->expectException(InvalidArgumentException::class);
        $this->expectExceptionMessage('end must be greater than or equal to 0');
        $this->iterator = $this->iterator->slice(1, -2);

        $this->expectException(InvalidArgumentException::class);
        $this->expectExceptionMessage('end must be greater than or equal to start');
        $this->iterator = $this->iterator->slice(3, 2);
    }

    public function testCollect(): void
    {
        $this->assertEquals([1, 2, 3, 4, 5], $this->iterator->collect());
    }

    public function testCount(): void
    {
        $this->assertEquals(5, $this->iterator->count());
        $this->iterator->rewind();
        $this->assertEquals(2, $this->iterator->count(fn($x) => $x % 2 === 0));
    }

    public function testAny(): void
    {
        $this->assertTrue($this->iterator->filter(fn($x) => $x === 5)->any());
        $this->assertFalse($this->iterator->any(fn($x) => $x === 3));

        $this->iterator->rewind();
        $this->assertTrue($this->iterator->any(fn($x) => $x === 3));
    }

    public function testAll(): void
    {
        $this->assertTrue($this->iterator->all(fn($x) => $x < 6));
        $this->iterator->rewind();
        $this->assertFalse($this->iterator->all(fn($x) => $x < 5));
    }

    public function testFind(): void
    {
        // TODO: improve test with an iterator containing duplicates
        $this->assertEquals(5, $this->iterator->find(fn($x) => $x === 5));
        $this->iterator->rewind();
        $this->assertNull($this->iterator->find(fn($x) => $x === 6));
    }

    public function testFindLast(): void
    {
        // TODO: improve test with an iterator containing duplicates
        $this->assertEquals(5, $this->iterator->findLast(fn($x) => $x === 5));
        $this->iterator->rewind();
        $this->assertNull($this->iterator->findLast(fn($x) => $x === 6));
    }

    public function testFirst(): void
    {
        $this->assertEquals(1, $this->iterator->first());
        $this->assertEquals(1, $this->iterator->first());
        $this->assertEquals(2, $this->iterator->skip(1)->first());
        $this->iterator->rewind();
        $this->assertEquals(1, $this->iterator->first());
    }

    public function testPosition(): void
    {
        $this->assertEquals(0, $this->iterator->position());
        $this->iterator->next();
        $this->assertEquals(1, $this->iterator->position());
        $this->iterator->rewind();
        $this->assertEquals(0, $this->iterator->position());
    }

    public function testChain(): void
    {
        $this->iterator = $this->iterator->chain(new ArrayIterator([6, 7, 8]));

        $this->assertEquals([1, 2, 3, 4, 5, 6, 7, 8], $this->iterator->collect());
    }

    public function testEnumerate(): void
    {
        $this->iterator = $this->iterator->enumerate();

        $this->assertEquals([[0, 1], [1, 2], [2, 3], [3, 4], [4, 5]], $this->iterator->collect());
    }

    public function testZip(): void
    {
        $this->iterator = $this->iterator->zip(new ArrayIterator([6, 7, 8]));

        $this->assertEquals([[1, 6], [2, 7], [3, 8], [4, null], [5, null]], $this->iterator->collect());
    }

    public function testInterleave(): void
    {
        $this->iterator = $this->iterator->interleave([6, 7, 8]);

        $this->assertEquals([1, 6, 2, 7, 3, 8, 4, 5], $this->iterator->collect());
    }

    public function testForEach(): void
    {
        $this->iterator->forEach(fn($x) => $this->assertEquals($x, $this->iterator->current()));
    }

    public function testFlatMap(): void
    {
        $this->iterator = $this->iterator->flatMap(fn($x) => [$x, $x]);

        $this->assertEquals([1, 1, 2, 2, 3, 3, 4, 4, 5, 5], $this->iterator->collect());
    }

    public function testFlatten(): void
    {
        $this->iterator = $this->iterator->map(fn($x) => [$x, $x])->flatten();

        $this->assertEquals([1, 1, 2, 2, 3, 3, 4, 4, 5, 5], $this->iterator->collect());
    }

    public function testGroupBy(): void
    {
        $this->iterator = $this->iterator->groupBy(fn($x) => $x % 2);

        $this->assertEquals([0 => [2, 4], 1 => [1, 3, 5]], $this->iterator->collect());
    }

    public function testInspect(): void
    {
        $count = 0;
        $this->iterator = $this->iterator->inspect(fn($x) => $count += $x);

        $this->assertEquals(5, $count);
        $this->assertEquals([1, 2, 3, 4, 5], $this->iterator->collect());
    }

    public function testCurrent(): void
    {
        $this->assertEquals(1, $this->iterator->current());
        $this->iterator->next();
        $this->assertEquals(2, $this->iterator->current());
        $this->iterator->rewind();
        $this->assertEquals(1, $this->iterator->current());
    }

    public function testNext(): void
    {
        $this->iterator->next();
        $this->assertEquals(2, $this->iterator->current());
        $this->iterator->next();
        $this->assertEquals(3, $this->iterator->current());
        $this->iterator->next();
        $this->assertEquals(4, $this->iterator->current());
        $this->iterator->next();
        $this->assertEquals(5, $this->iterator->current());
        $this->iterator->next();
        $this->assertNull($this->iterator->current());
    }

    public function testKey(): void
    {
        $this->assertEquals(0, $this->iterator->key());
        $this->iterator->next();
        $this->assertEquals(1, $this->iterator->key());
        $this->iterator->rewind();
        $this->assertEquals(0, $this->iterator->key());
    }

    public function testValid(): void
    {
        $this->assertTrue($this->iterator->valid());
        $this->iterator->next();
        $this->assertTrue($this->iterator->valid());
        $this->iterator->skip(3);
        $this->assertTrue($this->iterator->valid());
        $this->iterator->next();
        $this->assertFalse($this->iterator->valid());
    }

    public function testRewind(): void
    {
        $this->iterator->next();
        $this->iterator->next();
        $this->iterator->rewind();
        $this->assertEquals(1, $this->iterator->current());
    }
}
