# .TXT to .ARFF converter with Rust

===============================

This code was done for an university project on the subject of Data mining.For this project the task that was assigned to me was to convert the customerAge and customerBuys text file,**which follow the format specified below**,to .arff files, load them in WEKA and run FPgrowth and Apriori using WEKA.

**_NOTE:_** To avoid further problems the products were transliterated to english from greek (με απλα λογια τα εκανα greeklish)

### CustomerAge format

The format goes like this "customerID age".In this case I only want to view the distirbution of the customers age so _customerID_ will become a _string_ attribute and _age_ will become a _numeric_ attribute

### CustomerBuys format

The format is : "customerID DD/MM/YYYY product".In order to draw as many data as possible _customerID and the date and the products_ became _nominal_ attributes AND all of the _products_ were converted additionally to the _transaction format_ in order to be use FPgrowth and Apriori
