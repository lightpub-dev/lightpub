<?php
use Doctrine\ORM\Mapping\Entity;
use Doctrine\ORM\Mapping\Table;
use Doctrine\ORM\Mapping\Column;

#[Entity]
#[Table(name: 'user')]
class User
{
    #[Column(name: 'id', type: '')]
    private $id;
}