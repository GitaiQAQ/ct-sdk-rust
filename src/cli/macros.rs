// Copyright 2017 Gitai<i@gitai.me> All rights reserved.
//
// Permission is hereby granted, free of charge, to any person
// obtaining a copy of this software and associated documentation
// files (the "Software"), to deal in the Software without restriction,
// including without limitation the rights to use, copy, modify,
// merge, publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall
// be included in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES
// OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
// IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR
// ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF
// CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
// WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

macro_rules! titles {
    ($table:expr, $( $title:expr ), *) => {
        $table.set_titles(
            Row::new(vec![ $(
                Cell::new(stringify!($title).to_uppercase().as_ref()) ),* ]));
    };
}

macro_rules! table {
    ($table:expr, $( $title:expr ), *) => {
        titles!($table, $( $title ), *);
        $table.set_format(FormatBuilder::new()
            .padding(0, 5)
            .build());
        $table.printstd();
    };
}

/// Print struct by prettytable to std.
#[macro_export]
macro_rules! printstc {
    ($s:expr, $( $key:ident ), *) => ({
        let mut table = Table::init(vec![ $( Row::new(vec![
            Cell::new(stringify!($key).to_uppercase().as_ref()),
            Cell::new(format!("{}", $s.$key).as_ref())])) ,* ]);
        table.set_format(FormatBuilder::new()
            .padding(0, 5)
            .build());
        table.printstd();
    });
}

/// Print struct by prettytable to std.
#[macro_export]
macro_rules! printstd {
    ($s:expr, $( $key:ident ), *) => ({
        let mut table = Table::init($s
            .iter()
            .map(|elt|
                Row::new(vec![ $( Cell::new(format!("{}", elt.$key).as_ref()) ),* ]))
            .collect());
        table!(table, $( $key ), *);
    });
}

/// Print struct by prettytable to std.
#[macro_export]
macro_rules! printlist {
    ($s:expr, $key:ident) => ({
        $s
        .iter()
        .for_each(|elt|
            println!("{}", elt.$key));
    });
}

#[macro_export]
macro_rules! print_aws_err {
    ($error:expr) => ({
        debug!("{:#?}", $error);
        print!("{}", $error.aws.code);
    });
}
