grammar sillylolspeak;

lolcode : HAI comment* (head | variableDef)? body KBYE;
head	:MAEK_HEAD comment* variableDef? title MKAY;
title	:GIMMEH_TITLE TEXT+ OIC;
comment	:OBTW TEXT* TLDR;

body	:bodyItem*;
bodyItem:comment | paragraph | bold | list | newline | link | variableUse | TEXT;

paragraph: MAEK_PARAGRAF variableDef? paragraphItem* MKAY;
paragraphItem:comment | bold | list | newline | link | variableUse | TEXT;

bold	: GIMMEH_BOLD boldItem* OIC;
boldItem: variableUse | TEXT;

list	:MAEK_LIST variableDef? comment* listItem (listItem | comment)* MKAY;
listItem:	 GIMMEH_ITEM (bold | variableUse | TEXT)* OIC;

newline: NEWLINETAG;
link	:GIMMEH_LINX TEXT OIC;

variableDef:IHAZ TEXT ITIZ TEXT+ MKAY;
variableUse: GIMMEH_VAR TEXT OIC;


HAI 	:'#HAI' | '#hai';
KBYE 	:'#KBYE' | '#kbye';
OBTW 	:'#OBTW' | '#obtw';
TLDR 	:'#TLDR' | '#tldr';
OIC 	:'#OIC' | '#oic';
NEWLINETAG :'#NEWLINE' | '#newline';
IHAZ 	:'#IHAZ' | '#ihaz';
ITIZ 	:'#ITIZ' | '#itiz';
MKAY 	:'#MKAY' | '#mkay';


MAEK_HEAD :	 ('#MAEK' | '#maek') (' ' | '\t' | '\n' | '\r')+ ('HEAD' |'head');
MAEK_PARAGRAF :	 ('#MAEK' | '#maek') (' ' | '\t' | '\n' | '\r')+ ('PARAGRAF' |'paragraf');
MAEK_LIST :	 ('#MAEK' | '#maek') (' ' | '\t' | '\n' | '\r')+ ('LIST' |'list');

GIMMEH_TITLE :	 ('#GIMMEH' | '#gimmeh') (' ' | '\t' | '\n' | '\r')+ ('TITLE' |'title');
GIMMEH_BOLD :	 ('#GIMMEH' | '#gimmeh') (' ' | '\t' | '\n' | '\r')+ ('BOLD' |'bold');
GIMMEH_ITEM :	 ('#GIMMEH' | '#gimmeh') (' ' | '\t' | '\n' | '\r')+ ('ITEM' |'item');
GIMMEH_LINX :	 ('#GIMMEH' | '#gimmeh') (' ' | '\t' | '\n' | '\r')+ ('LINX' |'linx');
GIMMEH_VAR :	 ('#GIMMEH' | '#gimmeh') (' ' | '\t' | '\n' | '\r')+ ('VAR' |'var');

TEXT	:	('a'..'z' | 'A'..'Z' | '0'..'9' | ',' | '.' | '"' | ':' | '?' | '!' | '%' | '=' | '/')+;

EXTRAWHITESPACE:	(' ' | '\t' | '\r' | '\n')+ {skip();};

