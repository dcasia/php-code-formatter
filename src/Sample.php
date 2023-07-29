<?php

class Test
{
    function sample()
    {
        then(
            function () {
                $example = 1;
            },
            function () {
                $example = 1;
            }
        );
    }
}