<?php

declare(strict_types=1);

namespace Xenira\IterTools\Benchmarks;

use Xenira\IterTools\ArrayIterator;

class ArrayIteratorBench
{

    // /**
    //  * @Groups({"iter_tools"})
    //  * @Revs(500)
    //  * @Iterations(5)
    //  */
    // public function benchArrayIterator(): void
    // {
    //     // $rand = random_int(0, 10_000) * 2;
    //     $array = range(0, 10_000);
    //     // $search = array_rand($array);
    //     $arrayIterator = new ArrayIterator($array);
    //     // var_dump($arrayIterator);
    //     $arrayIterator = $arrayIterator->map(fn($value) => $value * 2);
    //     $arrayIterator = $arrayIterator->collect();
    //         // ->map(fn($value) => $value * 2)
    //         // ->count();
    //         // ->find(fn($value) => $value === $search);
    // }

    /**
     * @Groups({"IteRS"})
     * @Revs(500)
     * @Iterations(5)
     * @Assert("mode(variant.time.avg) < mode(baseline.time.avg) +/- 5%")
     */
    public function benchRustArrayIterator(): void {
        $array = range(0, 10_000);
        // $array = \map_array($array, fn($value) => $value * 2);
        $res =\Iter::new($array)->map(fn($value) => $value * 2)->collect();
    }

    // /**
    //  * @Groups({"iter_tools"})
    //  * @Revs(500)
    //  * @Iterations(5)
    //  */
    // public function benchIterTools(): void
    // {
    //     $array = range(0, 10_000);
    //     $array = \map_array($array, fn($value) => $value * 2);
    //     // $array = range(0, 10_000);
    //     // $arrayIterator = new \ArrayIter($array);
    //     // $arrayIterator = $arrayIterator->map(fn($value) => $value * 2);
    // }

    /**
     * @Groups({"vanilla"})
     * @Revs(500)
     * @Iterations(5)
     * @Assert("mode(variant.time.avg) < mode(baseline.time.avg) +/- 5%")
     */
    public function benchVanilla(): void
    {
        $array = range(0, 10_000);
        $array = array_map(fn ($value) => $value * 2, $array);
        // $array = array_sum($array);
        // sort($array);
        // $array = array_filter($array, function ($value) use ($search) {
        //     return $value === $search;
        // });
        // $cnt = count($array);
        // $array = array_search($search, $array);
        // $array = $array === false ? null : $array;
        // $array = array_search(fn($value) => $value % 2 === 0, $array);
    }


    /**
     * @Groups({"vanilla"})
     * @Revs(500)
     * @Iterations(5)
     */
    public function benchGenerator(): void {
        $array = range(0, 10_000);
        $array = $this->mapGenerator($array, fn($value) => $value * 2);
        $array = iterator_to_array($array);
    }

    public function mapGenerator($array, $callback) {
        foreach ($array as $value) {
            yield $callback($value);
        }
    }
    //
    // /**
    //  * @Groups({"vanilla_stupid"})
    //  * @Revs(1000)
    //  * @Iterations(5)
    //  */
    // public function benchVanillaStupid(): void
    // {
    //     $rand = random_int(0, 10000) * 2;
    //     $array = range(0, 10000);
    //     $search = array_rand($array);
    //     $array = array_map(fn($value) => $value * 2, $array);
    //     $array = array_filter($array, fn($value) => $value === $search);
    //     $array = array_shift($array);
    //     // $array = array_filter($array, fn($value) => $value % 2 !== 0);
    //     // if (count($array) <= 0) {
    //         // return;
    //     // }
    // }
}
