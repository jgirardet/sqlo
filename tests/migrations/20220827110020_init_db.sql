-- Add migration script here
PRAGMA foreign_keys = ON;


CREATE TABLE maison (
  id INTEGER NOT NULL PRIMARY KEY,
  adresse TEXT NOT NULL,
  taille INTEGER NOT NULL, 
  piscine BOOLEAN
);

CREATE TABLE piece (
  nb UUID NOT NULL PRIMARY KEY,
  lg INTEGER NOT NULL,
  la INTEGER NOT NULL,
  maison_id INTEGER NOT NULL,
  FOREIGN KEY(maison_id) REFERENCES maison(id)
);

CREATE TABLE adresse (
  id TEXT NOT NULL PRIMARY KEY,
  rue TEXT,
  m_id INTEGER NOT NULL,
  FOREIGN KEY(m_id) REFERENCES maison(id)
);

CREATE TABLE IF NOT EXISTS id_unique_int (
  id INTEGER NOT NULL PRIMARY KEY
);

CREATE TABLE IF NOT EXISTS id_unique_uuid (
  id UUID NOT NULL PRIMARY KEY
);


CREATE TABLE IF NOT EXISTS with_a_blob (
  id INTEGER NOT NULL PRIMARY KEY,
  data BLOB NOT NULL
);


INSERT INTO maison (id, adresse, taille) VALUES 
  (1, 'adresse1', 101),
  (2, 'adresse2', 102),
  (3, 'adresse3', 103);



INSERT INTO piece VALUES
  (X'11111111111111111111111111111111',1,10, 1),
  (X'22222222222222222222222222222222',2,20, 1),
  (X'33333333333333333333333333333333',3,30, 2),
  (X'44444444444444444444444444444444',4,40, 3),
  (X'55555555555555555555555555555555',5,50, 2),
  (X'66666666666666666666666666666666',6,60, 1),
  (X'77777777777777777777777777777777',7,70, 3),
  (X'88888888888888888888888888888888',8,80, 2),
  (X'99999999999999999999999999999999',9,90, 1);


INSERT INTO adresse (id, rue, m_id) VALUES 
  ('1', 'adresse1',1),
  ('2', 'adresse2',2),
  ('3', 'adresse3',3);
  

  CREATE TABLE IF NOT EXISTS lit ( 
    id INTEGER NOT NULL PRIMARY KEY,
    surface INTEGER NOT NULL
    ); 

INSERT INTO lit (id, surface) VALUES
  (1, 234),
  (2, 100),
  (3, 450),
  (4, 234);