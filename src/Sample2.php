<?php
declare(strict_types = 1);

namespace App\Console\Commands\Laravel;

use Illuminate\Console\Command;
use Illuminate\Encryption\Telescope;
use Illuminate\Filesystem\Filesystem;
use stdClass;
use Symfony\Component\Console\Command\Command as SymfonyCommand;
use LastOne\Throwable;

class EnvironmentEncryptCommand extends Filesystem
{
    public function handle(Throwable $throwable): int
    {
        $value = [1,2,3,new stdClass([1,2,3]),4];
    }
}
