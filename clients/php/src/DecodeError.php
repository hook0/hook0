<?php

declare(strict_types=1);

namespace Hook0;

/**
 * What the API answered is not what it declares it answers.
 *
 * Every strictness the runtime applies while reading a document reports this, so a caller has one
 * thing to catch whatever the shape of the answer was.
 */
final class DecodeError extends \RuntimeException
{
}
