from yahtzeevalue import Database


if __name__ == "__main__":
    import sys
    with Database(sys.argv[1]) as db:
        print(db.lookup(0))
        print(db.keep_first(0, [6, 6, 6, 6, 6, 1]))
        print(db.keep_second(0, [6, 6, 6, 6, 6, 1]))
        print(db.best_action(0, [6, 6, 6, 6, 6, 1]))
