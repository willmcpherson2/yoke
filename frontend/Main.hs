module Main where

import Lib

main :: IO ()
main = do
  project <- newProject
  unit <- newUnit project
  addData unit 0 0 0
  addMain unit
  printUnit unit
  freeUnit unit
  freeProject project
