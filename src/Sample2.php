<?php

declare(strict_types = 1);

namespace App\Console\Commands\Laravel;

use Illuminate\Encryption\Telescope;
use Illuminate\Filesystem\Filesystem;

use LastOne\Throwable;

class EnvironmentEncryptCommand extends Filesystem
{
    public function handle(Throwable $throwable): int
    {
        Telescope::hideRequestHeaders([ 'cookie', 'x-csrf-token', 'x-xsrf-token',  ]);
    }
}
