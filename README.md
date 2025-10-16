# JuMLi - Judge My List
A convenient database and modlist checking tool for the RimWorld modding community.

## Components
### jumli_data
Datasets used by jumli_gen. These are written in RON; here's an example:
```RON
Dataset(
    name: "Example Dataset",
    description: "Very descriptive!",

    records: [
        // Really terrible example mod
        (
            identifiers: [
                WorkshopId(1337),
                PackageId("really.terrible.example.mod")
            ],
            notices: [
                (
                    date: "1970-01-01",
                    notice: BadPerformance("Yuck!"), // ("explanation")
                    certainty: High, // How sure you are about this. Currently not shown to the user, but will be in the future.
                ),
                (
                    date: "1970-01-01",
                    notice: Unstable("Ewie!"), // ("explanation")
                    certainty: Medium,
                ),
                (
                    date: "1970-01-01",
                    notice: UseAlternative("Better Example Mod", 1338, "much nicer"), // ("name", workshop_id, "reason")
                    certainty: Medium,
                ),
                (
                    date: "1970-01-01",
                    notice: Miscellaneous("Hi mom!"), // ("note")
                    certainty: Low,
                ),
            ]
        )
    ]
)
```
### jumli_gen
A custom static site generator written in Rust. It uses data from Use This Instead and jumli_data to build an index for jumli_static and generate HTML/JSON mod reports.
### jumli_static
Static assets included with jumli_gen builds, including the list checking page and documentation.


## License
jumli_gen and jumli_static are made available under the terms of the GNU General Public License, version 3.

The datasets contained in jumli_data are available under the terms of the Creative Commons Zero 1.0 Universal license.
