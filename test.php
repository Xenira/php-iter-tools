<?php


$it = \ArrayIter::new([1, 2, 3, 4, 5])->chain(\ArrayIter::new([6, 7, 8, 9, 10]));
var_dump($it);
// $init = [ "foo" => 2 ];
// $ret = $it->scan($init, function($acc, $v) {
    // $acc["foo"] += $v;
//
//     if ($acc["foo"] > 10) {
//         return null;
//     }
//     return -$acc["foo"];
// })->collect();
        $result = 0;

        $it->forEach(function($x) use (&$result) {
            $result += $x;
        });
var_dump($result);
var_dump($it->collect());
// var_dump($it->collect());
//
$iterator = \ArrayIter::new([1, 2, 3, 4, 5]);
var_dump($iterator->sizeHint());
$iterator->next();
var_dump($iterator->sizeHint());
try {
    $iterator->map(fn($x) => throw new \Exception("test"))->collect();
} catch (\Exception $e) {
    var_dump($e->getMessage());
}

// array_map(fn($x) => throw new \Exception("test"), [1, 2, 3, 4, 5]);
testPartition();
testFuse();
testInspect();
testRfind();
    function testPartition(): void
    {
        $result = \ArrayIter::new([1, 2, 3, 4, 5])->partition(fn($x) => $x % 2 === 0);
        var_dump($result);
    }

 function testFuse(): void
    {
        $array = [1, 2, 3, null, 4, 5];
        $iterator = \ArrayIter::new($array);
        var_dump($iterator->fuse()->collect());
    }
function testInspect(): void
{
    $result = 0;
    $iterator = \ArrayIter::new([1, 2, 3, 4, 5]);

    $iterator->inspect(function ($x) use (&$result) {
      // var_dump($x);
      $result += $x;
    });

    var_dump($result);
    var_dump($iterator->collect());
    var_dump($result);
}
function testRfind()
    {
        $iter = \ArrayIter::new([1, 2, 3]);
        $result = $iter->rfind(fn($x) => $x === 2);
        var_dump($result);
        // $this->assertEquals(2, $result);
        // $this->assertEquals(1, $iter->nextBack());
    }
