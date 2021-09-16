#!/usr/bin/python


import sys


def main_fun():
    print 'Number of arguments:', len(sys.argv), 'arguments.'
    print 'Argument List:', str(sys.argv)

    # for aa in sys.argv:
    #     print aa

    sys.exit(0)


if __name__ == '__main__':
    main_fun()
