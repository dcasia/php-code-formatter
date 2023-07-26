<?php

class Test
{
    function sample1()
    {
        function sample2()
        {
            $hey2 = function () {
                match (true) {
                    default => function () {
                        return 1;
                    },
                };
            };
        }
    }
}
