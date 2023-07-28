<?php

class GroupCodeAttribute
{
    public static function make()
    {
static::one()
->a()->b(static::two()
->x()->y()
->z()->w())->c()->d();
    }
}
