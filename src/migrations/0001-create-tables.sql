CREATE TABLE `birbs`
(
	`id` INT NOT NULL AUTO_INCREMENT,
	`hash` BINARY(32) NOT NULL,
	`permalink` TINYTEXT NOT NULL,
	`source_url` VARCHAR(512) NOT NULL,
	`content_type` VARCHAR(64) NOT NULL,
	`banned` BOOLEAN NOT NULL DEFAULT FALSE,

	PRIMARY KEY (`id`),
	UNIQUE (`hash`),
	UNIQUE INDEX (`hash`)
);
