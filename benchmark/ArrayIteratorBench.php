<?php

declare(strict_types=1);

namespace Xenira\IterTools\Benchmarks;

use Xenira\IterTools\ArrayIterator;

class ArrayIteratorBench
{

    // /**
    //  * @Groups({"iter_tools"})
    //  * @Revs(1000)
    //  * @Iterations(5)
    //  */
    // public function benchArrayIterator(): void
    // {
    //     $rand = random_int(0, 100_000) * 2;
    //     $array = range(0, 10_000);
    //     $search = array_rand($array);
    //     $arrayIterator = new ArrayIterator($array);
    //     // $arrayIterator = $arrayIterator
    //         // ->map(fn($value) => $value * 2)
    //         // ->count();
    //         // ->find(fn($value) => $value === $search);
    // }

    /**
     * @Groups({"IteRS"})
     * @Revs(1000)
     * @Iterations(5)
     */
    public function benchRustArrayIterator(): void {
        $rand = random_int(0, 100_000);
        $array = range(0, 10_000);
        $search = array_rand($array);
        $arrayIterator = new \ArrayIter($array);
        // var_dump($arrayIterator);
        $arrayIterator = $arrayIterator->map(fn($value) => $value * 2);
        $arrayIterator = $arrayIterator->filter(fn($value) => $value % 2 === 0);
        $arrayIterator = $arrayIterator->chain(new \ArrayIter([1, 2, 3, 4, 5]));
        $arrayIterator = $arrayIterator->find(fn($value) => $value === $search);
    }

    /**
     * @Groups({"vanilla"})
     * @Revs(1000)
     * @Iterations(5)
     */
    public function benchVanilla(): void
    {
        $rand = random_int(0, 10000) * 2;
        $array = range(0, 100_000);
        $search = array_rand($array);
        $array = array_map(function ($value) {
            return $value * 2;
        }, $array);
        $array = array_filter($array, function ($value) use ($search) {
            return $value === $search;
        });
        $array = array_search($search, $array);
        $array = $array === false ? null : $array;
        // $array = array_search(fn($value) => $value % 2 === 0, $array);
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
