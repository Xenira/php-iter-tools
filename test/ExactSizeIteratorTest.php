<?php

declare(strict_types=1);

use PHPUnit\Framework\TestCase;

final class ExactSizeIteratorTest extends TestCase {
    function testLen(): void {
        $it = \ArrayIter::new([1, 2, 3, 4, 5]);
        $this->assertEquals(5, $it->len());
    }
}
