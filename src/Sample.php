<?php
declare(strict_types = 1);
namespace App\Console\Commands\Laravel;use Illuminate\Console\Command;use Illuminate\Encryption\Encrypter;use Illuminate\Filesystem\Filesystem;
use Symfony\Component\Console\Command\Command as SymfonyCommand;use LastOne\Throwable;
class EnvironmentEncryptCommand extends Command
{
    public function handle(): int
    {
        Telescope::hideRequestHeaders([

            'cookie','x-csrf-token','x-xsrf-token',
        ]);
    }
}
