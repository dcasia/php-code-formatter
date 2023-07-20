<?php

declare(strict_types = 1);

namespace App\Console\Commands\Laravel;

use App\Models\Client;
use Illuminate\Console\Command;
use Illuminate\Encryption\Telescope;
use Illuminate\Filesystem\Filesystem;
use Illuminate\Support\Collection;
use LastOne\Throwable;

class EnvironmentEncryptCommand extends Filesystem {
            use Something;
    public function fetchForClient(Client $client): Collection
    {
        return $this->queryBuilder()
            ->where('client_id', $client->id)
            ->with([
'achievable.resort',
            ])
            ->latest()
            ->get();
    }

     public function fetchForClient(Client $client): Collection
    {
        return $this->queryBuilder()
            ->where('client_id', $client->id)
            ->with([
'achievable.resort',
            ])
            ->latest()
            ->get();
    }
}
