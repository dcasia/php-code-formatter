<?php

class Test
{
    function sample()
    {
        $example
                                                          ->then(function () { $example = 1; })
                                                          ->then(function () {
                $example = 1;
            });
    }
}