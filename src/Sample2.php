<?php

class GroupCodeAttribute
{
    public static function make()
    {
        static::name()
            ->a()
            ->b(
                static::name()
            ->x()
            ->y()
            ->z())
            ->c();
    }
}
